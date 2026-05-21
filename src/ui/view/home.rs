use buoyant::{
    event::Event,
    focus::{self, FocusAction},
    view::{HStack, VStack, View, paginate::PageEvent, prelude::*},
};

use crate::{
    buttons_proto::Buttons,
    operation::{HeadlightMode, OperationCommand},
    ui::{
        colour::{self, ColorFormat},
        font, keys, state,
    },
};

#[must_use]
pub fn view(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    Paginate::new(
        focus::GROUP_0,
        |c, evt| {
            if let PageEvent::Next = evt {
                c.page_action = Some(state::PageAction::EnterSettings)
            }
        },
        Paginate::new(
            focus::GROUP_1,
            |c, evt| {
                c.next_operation_command = c.operation_state.as_active().map(|o| {
                    OperationCommand::SetSpeedMode(match evt {
                        buoyant::view::paginate::PageEvent::Next => o.speed_mode.increase(),
                        buoyant::view::paginate::PageEvent::Previous => o.speed_mode.decrease(),
                    })
                })
            },
            VStack::new((header(state), body(state).erase_captures())).with_spacing(4),
        ),
    )
    .map_event(|evt, _state: &mut ()| match evt {
        Event::KeyDown(keys::UP_CLICK) => Some(FocusAction::Next.into_event(focus::GROUP_1)),
        Event::KeyDown(keys::DOWN_CLICK) => Some(FocusAction::Previous.into_event(focus::GROUP_1)),
        // A bit hacky, but it's local
        Event::KeyDown(keys::CONFIRM_HOLD) => Some(FocusAction::Next.into_event(focus::GROUP_0)),
        _ => Some(evt.clone()),
    })
}

// Things we need to show:
//
// 1. [X] Time
// 2. [X] Speed mode
// 3. [X] Current speed
// 4. [X] Battery level
// 4.1 [X] Battery voltage/
// 5. [X] System current
// 6. [ ] Odometer - eventually
// 7. [X]Left/right blinker
// 8. [X] Headlight status
// 9. [X]Predicted range
// 10. [ ] Throttle

fn header(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    let speed_mode = state
        .operation_state
        .as_active()
        .map(|a| a.speed_mode.name())
        .unwrap_or("Unk");

    let flash_state = state
        .operation_state
        .as_active()
        .map(|a| match a.headlight_mode {
            HeadlightMode::Auto {
                currently_on: true, ..
            } => "☼!",
            HeadlightMode::Auto {
                currently_on: false,
                ..
            } => "☉!",
            HeadlightMode::On => "☼",
            HeadlightMode::Off => "☉",
        })
        .unwrap_or("☉");

    HStack::new((
        Text::new(speed_mode, &font::B612_REGULAR)
            .foreground_color(colour::ON_BACKGROUND)
            .flex_infinite_width(HorizontalAlignment::Leading),
        Text::new(flash_state, &font::ICONS)
            .foreground_color(colour::ON_BACKGROUND)
            .flex_infinite_width(HorizontalAlignment::Center),
        Text::new("Clock", &font::B612_REGULAR)
            .foreground_color(colour::ON_BACKGROUND)
            .flex_infinite_width(HorizontalAlignment::Trailing),
    ))
    .with_alignment(VerticalAlignment::Top)
    .padding(Edges::All, 10)
    .background_color(colour::BACKGROUND, Rectangle)
}

fn body(state: &state::State) -> impl View<ColorFormat, ()> + use<> {
    let left_blinker = state.system_state.buttons.contains(Buttons::L_BLINK);
    let right_blinker = state.system_state.buttons.contains(Buttons::R_BLINK);

    VStack::new((
        speedo(state).flex_frame().with_infinite_max_height(),
        HStack::new((
            VStack::new((
                half_infocard(
                    format_args!(
                        "{}.{}",
                        state.system_state.system_voltage.from_controller / 1000,
                        (state.system_state.system_voltage.from_controller / 100) % 10
                    ),
                    "V",
                    left_blinker,
                ),
                infocard(
                    state.system_state.battery_info.relative_soc as i16,
                    "% Battery",
                    left_blinker,
                ),
            ))
            .with_spacing(8),
            VStack::new((
                half_infocard(
                    format_args!(
                        "{}.{}",
                        (-state.system_state.battery_current / 1000) as i16,
                        (((-state.system_state.battery_current) / 100) % 10) as u8
                    ),
                    "A",
                    right_blinker,
                ),
                infocard(123_i16, "km Range", right_blinker),
            ))
            .with_spacing(8),
        ))
        .with_spacing(8)
        .flex_frame()
        // slightly annoying: padding doesn't seem to add to the size of the
        // container when we want it to be the size of its children. So we
        // need to bump the min height to ensure the container size is big
        // enough.
        .with_min_height(130 + 70 + 8 + 10)
        .padding(Edges::Bottom, 10),
    ))
    // .flex_infinite_width(HorizontalAlignment::Center)
    // .with_infinite_max_height()
}

fn speedo(state: &state::State) -> impl View<ColorFormat, ()> + use<> {
    HStack::new((
        Text::new_fmt::<10>(
            format_args!(
                "{}.{}",
                (state.system_state.motor_speed / 100) as u8,
                ((state.system_state.motor_speed / 10) % 10) as u8,
            ),
            &font::B612_REGULAR_VERY_LARGE_NUMBERS,
        )
        .with_font_size(2)
        .foreground_color(colour::ON_BACKGROUND),
        Text::new("km/h", &font::B612_REGULAR).foreground_color(colour::ON_BACKGROUND),
    ))
    .with_alignment(VerticalAlignment::Bottom)
    // We put a max width here so that a container width rectangle always draws
    // behind the text, preventing whole-screen redraws.
    .flex_infinite_width(HorizontalAlignment::Center)
    .background_color(colour::BACKGROUND, Rectangle)
}

fn half_infocard(
    args: core::fmt::Arguments,
    title: &'static str,
    blinker: bool,
) -> impl View<ColorFormat, ()> + use<> {
    let fg_colour = if blinker {
        colour::ON_TERTIARY
    } else {
        colour::ON_PRIMARY_CONTAINER
    };
    let bg_colour = if blinker {
        colour::TERTIARY
    } else {
        colour::PRIMARY_CONTAINER
    };

    HStack::new((
        Text::new_fmt::<8>(args, &font::B612_REGULAR_LARGE_NUMBERS).foreground_color(fg_colour),
        Text::new(title, &font::B612_REGULAR).foreground_color(fg_colour),
    ))
    .with_alignment(VerticalAlignment::Bottom)
    .with_spacing(8)
    .padding(Edges::Horizontal, 14)
    .frame_sized(130, 70)
    .background_color(bg_colour, RoundedRectangle::new(8))
}

fn infocard(value: i16, title: &'static str, blinker: bool) -> impl View<ColorFormat, ()> + use<> {
    let fg_colour = if blinker {
        colour::ON_TERTIARY
    } else {
        colour::ON_PRIMARY_CONTAINER
    };
    let bg_colour = if blinker {
        colour::TERTIARY
    } else {
        colour::PRIMARY_CONTAINER
    };

    VStack::new((
        Text::new_fmt::<4>(
            format_args!("{}", value),
            &font::B612_REGULAR_VERY_LARGE_NUMBERS,
        )
        .foreground_color(fg_colour),
        Text::new(title, &font::B612_REGULAR).foreground_color(fg_colour),
    ))
    .with_spacing(8)
    .frame_sized(130, 130)
    .background_color(bg_colour, RoundedRectangle::new(8))
}
