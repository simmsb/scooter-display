use at32f4xx_hal::can::{CanRx, CanTx, Frame, Id, filter::Mask32};

use crate::can_proto;

#[embassy_executor::task]
pub async fn can_rx(rx: CanRx<'static>) {
    can_rx_(rx).await;
}

async fn can_rx_(mut rx: CanRx<'static>) {
    let state_can_ch = crate::state::CAN_MESSAGES.sender();

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

        let Id::Standard(id) = msg.frame.id() else {
            defmt::info!("Unexpected extended can message: {}", msg);
            continue;
        };

        let Some(parsed) = can_proto::CanMessage::from_can_frame(id.as_raw(), msg.frame.data())
        else {
            defmt::info!("Unhandled CAN id: {}", id.as_raw());
            continue;
        };

        defmt::info!("Can RX: {}", msg);

        state_can_ch.send(parsed).await;
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
