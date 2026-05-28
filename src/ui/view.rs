use buoyant::{event::Event, match_view, view::prelude::*};
use embassy_time::Duration;

use crate::{can::LAST_SEEN_CAN_MESSAGE, operation::OperationCommand};

use super::{colour, keys};

pub mod home;
pub mod info;
pub mod locked;
pub mod settings;

use super::state::Page;

#[must_use]
pub fn root_view(state: &super::State) -> impl View<colour::ColorFormat, super::State> + use<> {
    match_view!((state.operation_state.is_locked(), state.page), {
        (true, _) => locked::view(state),
        (false, Page::Home) => home::view(state),
        (false, Page::Settings) => settings::view(state),
        (false, Page::Info) => info::view(state),
    })
    .padding(Edges::All, 5)
    .background_color(colour::BACKGROUND, RoundedRectangle::new(8))
    .captures_event(|e, s: &mut super::State| {
        if let Event::KeyDown(keys::POWER_HOLD) = e {
            let last_can_message = LAST_SEEN_CAN_MESSAGE.lock(|c| *c);

            // if we've not seen can messages recently, don't shut down from the
            // button. (So that powering on doesn't trigger a shutdown too)
            if last_can_message.elapsed() < Duration::from_secs(1) {
                crate::scram::trigger_controller_shutdown();
                let _ = s.next_operation_commands.push(OperationCommand::Lock);
            }
        }

        Some(e.clone())
    })
    // this padding might be needed if the display isn't full size
    // .padding(Edges::Top, 5)
}
