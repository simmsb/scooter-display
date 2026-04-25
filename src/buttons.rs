// - UART5
// - rx: GPIOD 2 (0x4) (`UART5_RX`)
// - tx: GPIOC 12 (0x1000) (`UART5_TX`)
// - 9600 baud

use at32f4xx_hal::{
    pac::UART5,
    uart::Serial5,
    uart::{Rx, Tx},
};
use deku::DekuContainerRead as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex, watch::Watch, zerocopy_channel};
use embassy_time::{Duration, Ticker};
use embedded_io_async::{Read as _, Write as _};
use static_cell::StaticCell;

use crate::buttons_proto::{self, ButtonParser};

// - if `BUTTON_UART_BUF[0] ^ BUTTON_UART_BUF[2] == 0xff` and `BUTTON_UART_BUF[1] ^ BUTTON_UART_BUF[3] == 0xff`
// - `BUTTON_DATA[0] = BUTTON_UART_BUF[0]`
// - `BUTTON_DATA[1] = BUTTON_UART_BUF[1]`

// ```
//   POWER_PRESSED = (BUTTON_DATA[0] & 1) == 0;
//   UP_PRESSED = (BUTTON_DATA[0] & 2) == 0;
//   DOWN_PRESSED = (BUTTON_DATA[0] & 4) == 0;
//   L_PRESSED = (BUTTON_DATA[0] & 8) == 0;
//   R_PRESSED = (BUTTON_DATA[1] & 1) == 0;
// ```

static BUTTON_STATE_WATCH: Watch<
    blocking_mutex::raw::CriticalSectionRawMutex,
    buttons_proto::Buttons,
    4,
> = Watch::new();

pub fn start_buttons(spawner: Spawner, uart: Serial5) {
    spawner.spawn(buttons_rx(uart).unwrap());
}

#[embassy_executor::task]
async fn buttons_rx(rx: Serial5) {
    buttons_rx_(rx).await;
}

async fn buttons_rx_(rx: Serial5) {
    let (_, mut rx) = rx.split();
    let sender = BUTTON_STATE_WATCH.sender();

    defmt::info!("buttons RX startup");
    loop {
        let mut buf = [0; 4];

        if let Err(_) = rx.read_exact(&mut buf).await {
            defmt::warn!("read error button rx");
            continue;
        };

        if (buf[0] ^ buf[2] != 0xff) || (buf[1] ^ buf[3] != 0xff) {
            defmt::warn!("buttons xor didn't match");
            continue;
        }

        let buttons = match ButtonParser::from_bytes((&buf[..2], 0)) {
            Ok(((_, _), buttons)) => buttons,
            Err(e) => {
                defmt::warn!(
                    "Button parse error: {} (in: {})",
                    defmt::Debug2Format(&e),
                    buf
                );
                continue;
            }
        };

        let buttons = buttons.as_buttons();

        sender.send_if_modified(|x| {
            if *x != Some(buttons) {
                *x = Some(buttons);

                true
            } else {
                false
            }
        });

        defmt::info!("Buttons RX: {} {}", buttons, buf);
    }
}
