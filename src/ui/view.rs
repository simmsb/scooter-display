use buoyant::{event::Event, match_view, view::prelude::*};

use crate::{operation::OperationCommand, platform};

use super::{colour, keys, state::State};

pub mod home;
pub mod info;
pub mod locked;
pub mod settings;

use super::state::Page;

#[must_use]
pub fn root_view(state: &State) -> impl View<colour::ColorFormat, State> + use<> {
    match_view!((state.operation_state.is_locked(), state.page), {
        (true, _) => locked::view(state),
        (false, Page::Home) => home::view(state),
        (false, Page::Settings) => settings::view(state),
        (false, Page::Info) => info::view(state),
    })
    .padding(Edges::All, 5)
    .background_color(colour::background(), Rectangle)
    .captures_event(|e, s: &mut State| {
        if let Event::KeyDown(keys::POWER_HOLD) = e {
            // if we've not seen can messages recently, don't shut down from the
            // button. This prevents shutdown firing when using the power button
            // to start up.
            if platform::can_is_alive() {
                platform::trigger_shutdown();
                let _ = s.next_operation_commands.push(OperationCommand::Lock);
            }
        }

        Some(e.clone())
    })
    // this padding might be needed if the display isn't full size
    // .padding(Edges::Top, 5)
}
