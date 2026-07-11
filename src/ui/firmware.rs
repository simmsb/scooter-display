use buoyant::{
    app::{App, Harness as _},
    focus::Role,
    render_target::{EmbeddedGraphicsRenderTarget, RenderTarget},
};
use embassy_futures::select;
use embassy_time::{Duration, Instant, Ticker};

use super::engine::UiEngine;

use crate::{
    buttons::{BHDuration, BHInstant, BUTTON_EVENTS, Button},
    operation,
    system_state,
};

#[embassy_executor::task]
pub async fn ui(display: crate::display::Display) {
    ui_(display).await;
}

fn map_event(
    (button, evt): (Button, butt_head::Event<BHDuration, BHInstant>),
) -> Option<(buoyant::event::Event, buoyant::event::Event)> {
    let (key, key_hold) = match button {
        Button::Up => (super::keys::UP_CLICK, super::keys::UP_HOLD),
        Button::Down => (super::keys::DOWN_CLICK, super::keys::DOWN_HOLD),
        Button::Confirm => (super::keys::CONFIRM_CLICK, super::keys::CONFIRM_HOLD),
        Button::Power => (super::keys::POWER_CLICK, super::keys::POWER_HOLD),
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

async fn ui_(display: crate::display::Display) {
    let (mut display, mut backlight) = display.split();
    let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display, super::colour::black());

    let app_start = Instant::now();

    let app = static_cell::make_static!(
        App::new(super::state::State::new(), target.size(), super::view::root_view)
            .with_roles(Role::Button | Role::Container)
    );

    app.focus_forward();

    target.clear(super::colour::black());

    let mut ui_engine = UiEngine::new();

    defmt::debug!("UI APP size: {}", core::mem::size_of_val(app));

    let mut immediate_redraw = false;

    let mut button_events = BUTTON_EVENTS.subscriber().unwrap();
    let mut op_state_updates = operation::STATE_UPDATES.receiver().unwrap();
    let mut sys_state_updates = system_state::STATE_UPDATES.receiver().unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(500));

    if crate::ON_BENCH {
        backlight.backlight_enable(true);
    }

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

        let last_can_message = crate::can::LAST_SEEN_CAN_MESSAGE.lock(|c| *c);
        defmt::trace!("Last seen can: {}", last_can_message);
        defmt::trace!("Last seen can el: {}", last_can_message.elapsed());

        if !crate::ON_BENCH {
            if last_can_message.elapsed() > Duration::from_secs(4) {
                backlight.backlight_enable(false);
            } else {
                backlight.backlight_enable(true);
            }
        }

        let mut events = heapless::Vec::<buoyant::event::Event, 8, u8>::new();

        if let Some((a, b)) = event.and_then(map_event) {
            let _ = events.push(a);
            let _ = events.push(b);
        }

        while let Some(event) = button_events.try_next_message_pure() {
            if let Some((a, b)) = map_event(event) {
                let _ = events.push(a);
                let _ = events.push(b);
            }
        }

        let tick_result = ui_engine.tick(
            app,
            &mut target,
            app_start.elapsed().into(),
            events.into_iter(),
        );

        for op_command in tick_result.operation_commands {
            defmt::trace!("op command: {}", op_command);
            operation::OPERATION_COMMANDS.send(op_command).await;
        }

        immediate_redraw = tick_result.rendered;
    }
}
