use buoyant::{event::Event, match_view, view::prelude::*};

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
    .captures_event(|e, _s: &mut super::State| {
        if !crate::ON_BENCH
            && let Event::KeyDown(keys::POWER_HOLD) = e
        {
            crate::scram::scram();
        }

        Some(e.clone())
    })
    // this padding might be needed if the display isn't full size
    // .padding(Edges::Top, 5)
}
