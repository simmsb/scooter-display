use buoyant::{match_view, view::prelude::*};

use super::colour;

pub mod home;
pub mod locked;
pub mod settings;

use super::state::Page;

#[must_use]
pub fn root_view(state: &super::State) -> impl View<colour::ColorFormat, super::State> + use<> {
    match_view!((state.operation_state.is_locked(), state.page), {
        (true, _) => locked::view(state),
        (false, Page::Home) => home::view(state),
        (false, Page::Settings) => EmptyView,
    })
    .background_color(colour::BACKGROUND, Rectangle)
    .padding(Edges::All, 5)
}
