use buoyant::{
    app::Harness as _,
    focus::Role,
    render_target::{EmbeddedGraphicsRenderTarget, RenderTarget},
    view::ViewLayout,
};
use embassy_futures::select;
use embassy_time::{Duration, Instant, Ticker};

use crate::{
    buttons::{BHDuration, BHInstant, BUTTON_EVENTS, Button},
    operation::{self},
    system_state,
};

use self::state::State;

pub mod colour;
pub mod font;
pub mod state;
pub mod view;

#[embassy_executor::task]
pub async fn ui(display: crate::display::Display) {
    ui_(display).await;
}

const fn root_view_differ_size<V, T, S>(_f: fn(T) -> V) -> usize
where
    V: ViewLayout<S>,
    V::Renderables: buoyant::render::Diffable,
{
    use buoyant::render::Diffable;

    V::Renderables::SIZE.div_ceil(8) + 1
}

/// buoyant uses keyboard-like key events, but we have a keypad with up/down/confirm/power.
///
/// We also want to make use of hold events.
///
/// So translate our click/hold events into arbitrary buoyant character presses.
///
/// UI views will use map_event to translate these into the correct focus action.
mod keys {
    use buoyant::event::Key;

    pub const UP_CLICK: Key = Key::Character('0');
    pub const UP_HOLD: Key = Key::Character('1');
    pub const DOWN_CLICK: Key = Key::Character('2');
    pub const DOWN_HOLD: Key = Key::Character('3');
    pub const CONFIRM_CLICK: Key = Key::Character('4');
    pub const CONFIRM_HOLD: Key = Key::Character('5');
    pub const POWER_CLICK: Key = Key::Character('6');
    pub const POWER_HOLD: Key = Key::Character('7');
}

fn map_event(
    (button, evt): (Button, butt_head::Event<BHDuration, BHInstant>),
) -> Option<(buoyant::event::Event, buoyant::event::Event)> {
    let (key, key_hold) = match button {
        Button::Up => (keys::UP_CLICK, keys::UP_HOLD),
        Button::Down => (keys::DOWN_CLICK, keys::DOWN_HOLD),
        Button::Confirm => (keys::CONFIRM_CLICK, keys::CONFIRM_HOLD),
        Button::Power => (keys::POWER_CLICK, keys::POWER_HOLD),
    };

    match evt {
        butt_head::Event::Press { .. } => None,
        butt_head::Event::Release { .. } => None,
        butt_head::Event::Click { .. } => Some((
            buoyant::event::Event::KeyDown(key),
            buoyant::event::Event::KeyUp(key),
        )),
        butt_head::Event::Hold { .. } => Some((
            buoyant::event::Event::KeyDown(key_hold),
            buoyant::event::Event::KeyUp(key_hold),
        )),
    }
}

async fn ui_(mut display: crate::display::Display) {
    let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display.inner, colour::BLACK);

    let app_start = Instant::now();

    let app = static_cell::make_static!(
        buoyant::app::App::new(state::State::new(), target.size(), view::root_view)
            .with_roles(Role::Button | Role::Container)
    );

    app.focus_forward();

    target.clear(colour::BLACK);

    let mut diffing_mem = [0u8; root_view_differ_size(view::root_view)];

    defmt::debug!("UI APP size: {}", core::mem::size_of_val(app));
    defmt::trace!("UI differ takes {} bytes", diffing_mem.len());

    let mut immediate_redraw = false;

    let mut button_events = BUTTON_EVENTS.subscriber().unwrap();
    let mut op_state_updates = operation::STATE_UPDATES.receiver().unwrap();
    let mut sys_state_updates = system_state::STATE_UPDATES.receiver().unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        let event = if !immediate_redraw {
            match select::select4(
                button_events.next_message_pure(),
                op_state_updates.changed(),
                sys_state_updates.changed(),
                ticker.next(),
            )
            .await
            {
                select::Either4::First(but) => Some(but),
                select::Either4::Second(_) => {
                    operation::read_state(|s| {
                        if &app.state().operation_state != s {
                            defmt::trace!("Doing operation state copy");
                            app.state_mut().operation_state.clone_from(s)
                        }
                    });
                    None
                }
                select::Either4::Third(_) => {
                    system_state::read_state(|s| {
                        if &app.state().system_state != s {
                            defmt::trace!("Doing system state copy: {}", s);
                            app.state_mut().system_state.clone_from(s);
                        }
                    });
                    None
                }
                select::Either4::Fourth(_) => None,
            }
        } else {
            None
        };

        let start = Instant::now();

        if let Some((a, b)) = event.and_then(map_event) {
            app.send(a);
            app.send(b);
        }

        while let Some(event) = button_events.try_next_message_pure() {
            if let Some((a, b)) = map_event(event) {
                app.send(a);
                app.send(b);
            }
        }

        app.set_time(app_start.elapsed().into());

        if let Some(action) = app.state().page_action {
            let current_page = app.state().page;
            let new_page = current_page.handle_action(action);
            let mut state = app.state_mut();

            if let Some(new_page) = new_page {
                state.page = new_page;
                state.page_action = None;
            }

            defmt::trace!("Page: {}", state.page);
        }

        if !app.state().next_operation_commands.is_empty() {
            for op_command in app.state_mut().next_operation_commands.drain(..) {
                defmt::trace!("op command: {}", op_command);
                operation::OPERATION_COMMANDS.send(op_command).await;
            }
        }
        if app.should_redraw() || target.clear_animation_status() {
            defmt::trace!("Redrawing");

            // target.clear(Rgb565::RED);
            // let _start = Instant::now();
            app.render_animated_diffed(&mut target, &colour::BLACK, &mut diffing_mem);
            // app.render_only_target(&mut target, &colour::BACKGROUND);
            // app.draw_focus_overlay(&mut target, colour::ColorFormat::RED, 2);

            immediate_redraw = true;

            defmt::trace!("Display draw took {}ms", start.elapsed().as_millis());
        } else {
            immediate_redraw = false;
        }
    }
}
