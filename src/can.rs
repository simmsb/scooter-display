use at32f4xx_hal::can::{CanRx, CanTx, Frame, Id, filter::Mask32};
use embassy_executor::SendSpawner;
use embassy_futures::select;
use embassy_time::{Duration, Instant};

use crate::{
    can_proto::{self, CanMessage, DisplayChargeHistoryRequest, TriggerUpdate},
    no_inline_future::NoInlineFutExt,
    scram,
};

pub static CAN_TX_BUS: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    can_proto::Sent,
    1,
> = embassy_sync::channel::Channel::new();

pub fn start_can(spawner: SendSpawner, tx: CanTx<'static>, rx: CanRx<'static>) {
    spawner.spawn(can_rx(rx).unwrap());
    spawner.spawn(can_tx(tx).unwrap());
    spawner.spawn(can_periodic().unwrap());
}

pub static LAST_SEEN_CAN_MESSAGE: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    Instant,
> = embassy_sync::blocking_mutex::Mutex::new(Instant::MIN);

#[embassy_executor::task]
async fn can_rx(rx: CanRx<'static>) {
    can_rx_(rx).await;
}

async fn can_rx_(mut rx: CanRx<'static>) {
    let state_can_ch = crate::system_state::CAN_MESSAGES.sender();

    defmt::info!("Can RX startup");

    rx.modify_filters()
        .enable_bank(0, at32f4xx_hal::can::Fifo::Fifo0, Mask32::accept_all())
        .enable_bank(1, at32f4xx_hal::can::Fifo::Fifo1, Mask32::accept_all());

    let mut errors = 0;

    loop {
        let msg = match rx.read().no_inline().await {
            Ok(msg) => msg,
            Err(at32f4xx_hal::can::enums::BusError::BitDominant) => {
                // bitdominant happens when controller is not powered
                continue;
            }
            Err(e) => {
                defmt::error!("Can read err: {}", e);
                errors += 1;

                if errors > 10 {
                    defmt::error!(
                        "Sleeping can task due to errors, probably due to can not being connected"
                    );
                    embassy_time::Timer::after_secs(1).await;
                }
                continue;
            }
        };

        let now = Instant::now();
        unsafe {
            LAST_SEEN_CAN_MESSAGE.lock_mut(|t| *t = now);
        }

        errors = 0;

        let id = match msg.frame.id() {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        };

        let parsed = match can_proto::CanMessage::from_can_frame(id, msg.frame.data()) {
            Ok(Some(parsed)) => parsed,
            Ok(None) => {
                defmt::trace!("Unhandled CAN id ({}): {}", id, msg.frame.data());
                continue;
            }
            Err(reason) => {
                defmt::error!(
                    "Failed to parse CAN id ({}): {} ({})",
                    id,
                    msg.frame.data(),
                    defmt::Debug2Format(&reason)
                );
                continue;
            }
        };

        defmt::trace!("Can RX: {}", parsed);

        // if we see the update message, reboot so the bootloader can take over
        // a firmware update
        if let CanMessage::TriggerUpdate(TriggerUpdate { .. }) = &parsed {
            scram::scram();
        }

        state_can_ch.send(parsed).no_inline().await;
    }
}

#[embassy_executor::task]
async fn can_tx(tx: CanTx<'static>) {
    can_tx_(tx).await;
}

async fn can_tx_(mut tx: CanTx<'static>) {
    defmt::info!("Can TX startup");

    let can_tx_ch = CAN_TX_BUS.receiver();

    let mut buf: [u8; 8] = [0u8; 8];

    loop {
        let to_send = can_tx_ch.receive().no_inline().await;
        let buf = match to_send.serialise(&mut buf) {
            Ok(buf) => buf,
            Err(e) => {
                defmt::error!("Couldn't serialise can frame: {}", defmt::Debug2Format(&e));
                continue;
            }
        };

        let frame = if to_send.can_id().is_extended() {
            match Frame::new_extended(to_send.can_id().to_extended_raw(), buf) {
                Ok(f) => f,
                Err(e) => {
                    defmt::error!("Couldn't create frame: {}", e);
                    continue;
                }
            }
        } else {
            match Frame::new_standard(to_send.can_id().to_standard_raw(), buf) {
                Ok(f) => f,
                Err(e) => {
                    defmt::error!("Couldn't create frame: {}", e);
                    continue;
                }
            }
        };

        defmt::trace!("Can TX ({}): {}", to_send.can_id(), buf);

        let _txstatus = tx.write(&frame).no_inline().await;
    }
}

#[embassy_executor::task]
async fn can_periodic() {
    can_periodic_().await;
}

async fn can_periodic_() {
    defmt::info!("Can Periodic startup");

    let can_tx = CAN_TX_BUS.sender();

    let mut request_battery_charge_history_ticker =
        embassy_time::Ticker::every(Duration::from_secs(60));

    // just so we have the code structure for N tickers
    let mut todo_ticker = embassy_time::Ticker::every(Duration::from_secs(10000));

    loop {
        match select::select(
            request_battery_charge_history_ticker.next(),
            todo_ticker.next(),
        )
        .no_inline()
        .await
        {
            select::Either::First(_) => {
                can_tx
                    .send(can_proto::Sent::DisplayChargeHistoryRequest(
                        DisplayChargeHistoryRequest,
                    ))
                    .no_inline()
                    .await;
            }
            select::Either::Second(_) => {}
        }
    }
}
