use buoyant::{
    event::{Event, Key},
    focus::{self, FocusAction},
    match_view,
    view::{
        HStack, Rotary, VStack, View,
        prelude::*,
        rotary::{RotaryEvent, RotaryState},
    },
};

use crate::ui::{
    colour::{self, ColorFormat},
    font, keys, state,
};

#[must_use]
pub fn view(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    VStack::new((header(state), body(state)))
        .with_spacing(2)
        .with_alignment(HorizontalAlignment::Center)
        .flex_infinite_width(HorizontalAlignment::Center)
        .with_infinite_max_height()
        .focus_touches()
        .map_event(|event, _: &mut ()| match event {
            Event::KeyDown(key) => match *key {
                keys::UP_CLICK => Some(FocusAction::Previous.into_event(focus::GROUP_0)),
                keys::DOWN_CLICK => Some(FocusAction::Next.into_event(focus::GROUP_0)),
                keys::CONFIRM_CLICK => Some(FocusAction::Select.into_event(focus::GROUP_0)),
                _ => None,
            },
            Event::KeyUp(_) => None,
            _ => Some(event.clone()),
        })
}

// Things we need to show:
//
// 1. Time
// 2. Speed mode
// 3. Current speed
// 4. Battery level (inc voltage)
// 5. System current
// 6. Odometer - eventually
// 7. Left/right blinker

fn header(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    HStack::new((
        Text::new("Drive mode", &font::B612_REGULAR)
            .multiline_text_alignment(HorizontalTextAlignment::Leading)
            .foreground_color(colour::ON_BACKGROUND),
        Text::new("Clock", &font::B612_REGULAR)
            .multiline_text_alignment(HorizontalTextAlignment::Center)
            .foreground_color(colour::ON_BACKGROUND),
    ))
    .with_alignment(VerticalAlignment::Top)
    .flex_infinite_width(HorizontalAlignment::Center)
    .with_infinite_max_height()
}

fn body(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    VStack::new((speedo(state),))
        .with_spacing(4)
        .with_alignment(HorizontalAlignment::Center)
        .flex_infinite_width(HorizontalAlignment::Center)
        .with_infinite_max_height()
}

fn speedo(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    Text::new_fmt::<4>(
        format_args!("{}", state.system_state.motor_speed),
        &font::B612_REGULAR_VERY_LARGE_NUMBERS,
    )
    .multiline_text_alignment(HorizontalTextAlignment::Center)
    .foreground_color(colour::ON_BACKGROUND)
}
