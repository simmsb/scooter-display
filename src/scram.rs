use crate::can_proto::{self, CanValue};
use at32f4xx_hal::can::{CanTx, Frame};
use deku::DekuContainerWrite as _;

pub fn trigger_controller_shutdown() {
    // The CAN bus should already be initialised, so just summon the rust
    // wrapper.
    //
    // If the CAN bus isn't initialised, then we can't have been
    // sending messages to the controller.
    //
    // In such a case this will probably hang, or maybe fail silently, either
    // way, nothing nasty can happen.
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
        let _ = can_tx.try_write(&frame);
        cortex_m::asm::delay(500_000);
    }
}

pub fn scram() -> ! {
    trigger_controller_shutdown();
    cortex_m::peripheral::SCB::sys_reset();
}
