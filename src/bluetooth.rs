// - USART2
// - rx: GPIOA 3 (bit 0x8) (`USART2_RX`)
// - tx: GPIOA 2 (bit 0x4) (`USART2_TX`)
// - 57600 baud

use at32f4xx_hal::{
    pac::USART2,
    uart::{Rx, Tx},
};
use embedded_io_async::{Read, Write};

#[embassy_executor::task]
pub async fn bluetooth_rx(rx: Rx<USART2, u8>) {
    bluetooth_rx_(rx).await;
}

async fn bluetooth_rx_(mut rx: Rx<USART2, u8>) {
    defmt::info!("bluetooth RX startup");
    loop {
        let mut buf = [0; 16];
        if let Err(_e) = rx.read(&mut buf).await {
            defmt::error!("Bluetooth read err");
            continue;
        };

        defmt::info!("Bluetooth RX: {}", buf);
    }
}

#[embassy_executor::task]
pub async fn bluetooth_tx(tx: Tx<USART2, u8>) {
    bluetooth_tx_(tx).await;
}

async fn bluetooth_tx_(mut tx: Tx<USART2, u8>) {
    defmt::info!("Bluetooth TX startup");
    // loop {
    //     embassy_time::Timer::after_secs(1).await;
    // }
}
