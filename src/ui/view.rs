use buoyant::{match_view, view::prelude::*};

use super::colour;

pub mod home;
pub mod locked;
pub mod settings;

use super::state::Page;

#[must_use]
pub fn root_view(state: &super::State) -> impl View<colour::ColorFormat, super::State> + use<> {
    match_view!(state.page, {
        Page::Locked => locked::view(state),
        Page::Home => home::view(state),
        Page::Settings => EmptyView,
    })
    .background_color(colour::BACKGROUND, Rectangle)
    .padding(Edges::All, 5)
}
