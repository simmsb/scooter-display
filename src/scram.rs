use crate::can_proto::{self, CanValue};
use at32f4xx_hal::can::{CanTx, Frame};
use deku::DekuContainerWrite as _;

pub fn scram() -> ! {
    // the CAN bus should already be initialised, so just summon the rust wrapper.
    let mut can_tx = unsafe { core::mem::conjure_zst::<CanTx<'static>>() };

    let mut buf = [0u8; 8];

    let msg = can_proto::DisplaySpeedMode::shutdown();
    if msg.to_slice(&mut buf).is_err() {
        cortex_m::peripheral::SCB::sys_reset();
    }

    let Ok(frame) = Frame::new_standard(
        can_proto::DisplaySpeedMode::can_id().to_standard_raw(),
        buf.as_slice(),
    ) else {
        cortex_m::peripheral::SCB::sys_reset();
    };

    for _ in 0..8u8 {
        embassy_futures::block_on(can_tx.write(&frame));
    }

    cortex_m::peripheral::SCB::sys_reset();
}
