use at32f4xx_hal::can::{CanRx, CanTx, Frame, filter::Mask32};

#[embassy_executor::task]
pub async fn can_rx(rx: CanRx<'static>) {
    can_rx_(rx).await;
}

async fn can_rx_(mut rx: CanRx<'static>) {
    defmt::info!("Can RX startup");
    rx.modify_filters()
        .enable_bank(0, at32f4xx_hal::can::Fifo::Fifo0, Mask32::accept_all())
        .enable_bank(1, at32f4xx_hal::can::Fifo::Fifo1, Mask32::accept_all());
    loop {
        let msg = match rx.read().await {
            Ok(msg) => msg,
            Err(e) => {
                defmt::error!("Can read err: {}", e);
                continue;
            }
        };

        defmt::info!("Can RX: {}", msg);
    }
}

#[embassy_executor::task]
pub async fn can_tx(tx: CanTx<'static>) {
    can_tx_(tx).await;
}

async fn can_tx_(mut tx: CanTx<'static>) {
    defmt::info!("Can TX startup");
    loop {
        embassy_time::Timer::after_secs(1).await;

        let frame = Frame::new_standard(0x12, &[0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        let _txstatus = tx.write(&frame).await;
        defmt::trace!("tx");
    }
}
