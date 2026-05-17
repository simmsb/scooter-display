// - UART5
// - rx: GPIOD 2 (0x4) (`UART5_RX`)
// - tx: GPIOC 12 (0x1000) (`UART5_TX`)
// - 9600 baud

use at32f4xx_hal::{exti::ExtiInput, gpio::Pin, uart::Serial5};
use deku::DekuContainerRead as _;
use embassy_executor::Spawner;
use embassy_futures::select::{self, select};
use embassy_sync::{blocking_mutex, watch::Watch};
use embassy_time::{Duration, WithTimeout};
use embedded_io_async::Read as _;

use crate::buttons_proto::{self, ButtonParser, Buttons};

pub static BUTTON_STATE_WATCH: Watch<
    blocking_mutex::raw::ThreadModeRawMutex,
    buttons_proto::Buttons,
    4,
> = Watch::new();

pub fn start_buttons(spawner: Spawner, uart: Serial5, power_button: ExtiInput<Pin<'A', 1>, 1>) {
    spawner.spawn(buttons_rx(uart, power_button).unwrap());
}

#[embassy_executor::task]
async fn buttons_rx(rx: Serial5, power_button: ExtiInput<Pin<'A', 1>, 1>) {
    buttons_rx_(rx, power_button).await;
}

async fn buttons_rx_(rx: Serial5, mut power_button: ExtiInput<Pin<'A', 1>, 1>) {
    let (_, mut rx) = rx.split();
    let sender = BUTTON_STATE_WATCH.sender();

    defmt::info!("buttons RX startup");

    let mut buttons = Buttons::empty();

    loop {
        let mut buf = [0; 4];

        match select(rx.read_exact(&mut buf), power_button.wait_for_any_edge()).await {
            select::Either::First(r) => {
                if r.is_err() {
                    // XXX: this error case probably doesn't happen
                    defmt::warn!("read error button rx");
                    continue;
                };

                if (buf[0] ^ buf[2] != 0xff)
                    || (buf[1] ^ buf[3] != 0xff)
                    || (buf[0] & 0b11000000 != 0)
                    || (buf[1] & 0b11111110 != 0)
                {
                    defmt::warn!("buttons xor didn't match, or invalid: {}", buf);

                    // on framing error, read until we have nothing
                    let _ = async {
                        for _ in 0..4 {
                            let _ = rx.read_exact(&mut buf[..1]).await;
                        }
                    }
                    .with_timeout(Duration::from_millis(50))
                    .await;

                    continue;
                }

                let parsed = match ButtonParser::from_bytes((&buf[..2], 0)) {
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

                buttons.update_from_uart(parsed);
            }
            select::Either::Second(_) => {
                // TODO: debounce
                buttons.set(Buttons::POWER, power_button.is_low());
            }
        }

        sender.send_if_modified(|x| {
            if *x != Some(buttons) {
                *x = Some(buttons);

                true
            } else {
                false
            }
        });

        defmt::info!("Buttons update: {}", defmt::Debug2Format(&buttons));
    }
}
