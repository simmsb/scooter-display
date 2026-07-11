use buoyant::{
    app::{App, Harness as _},
    event::Event,
    render_target::RenderTarget,
    view::View,
};
use chrono::Timelike as _;

use super::{colour, state, view};
use crate::operation::OperationCommand;

const fn root_view_differ_size<V, T, S>(_f: fn(T) -> V) -> usize
where
    V: buoyant::view::ViewLayout<S>,
    V::Renderables: buoyant::render::Diffable,
{
    use buoyant::render::Diffable;

    V::Renderables::SIZE.div_ceil(8) + 1
}

const DIFF_SIZE: usize = root_view_differ_size(view::root_view);

pub struct TickResult {
    pub rendered: bool,
    pub operation_commands: heapless::Vec<OperationCommand, 3, u8>,
}

pub struct UiEngine {
    diffing_mem: [u8; DIFF_SIZE],
}

impl UiEngine {
    pub const fn new() -> Self {
        Self {
            diffing_mem: [0u8; DIFF_SIZE],
        }
    }

    pub fn tick<V, F, S>(
        &mut self,
        app: &mut App<V, state::State, F>,
        target: &mut S,
        elapsed: core::time::Duration,
        events: impl IntoIterator<Item = Event>,
    ) -> TickResult
    where
        V: View<colour::ColorFormat, state::State>,
        F: Fn(&state::State) -> V,
        S: RenderTarget<ColorFormat = colour::ColorFormat>,
    {
        for event in events {
            app.send(event);
        }

        if let Some(action) = app.state().page_action {
            let current_page = app.state().page;
            if let Some(new_page) = current_page.handle_action(action) {
                let mut ui_state = app.state_mut();
                ui_state.page = new_page;
                ui_state.page_action = None;
            }
        }

        let mut operation_commands = heapless::Vec::new();
        if !app.state().next_operation_commands.is_empty() {
            for op_command in app.state_mut().next_operation_commands.drain(..) {
                let _ = operation_commands.push(op_command);
            }
        }

        app.set_time(elapsed);

        let now = crate::platform::current_time();
        let hour = now.hour() as u8;
        let minute = now.minute() as u8;
        let second = now.second() as u8;

        if app.state().hour != hour || app.state().minute != minute || app.state().second != second {
            let mut state = app.state_mut();
            state.hour = hour;
            state.minute = minute;
            state.second = second;
        }

        let rendered = if app.should_redraw() {
            app.render_animated_diffed(target, &colour::black(), &mut self.diffing_mem);
            true
        } else {
            false
        };

        TickResult {
            rendered,
            operation_commands,
        }
    }
}
