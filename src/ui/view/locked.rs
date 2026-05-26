use buoyant::{
    event::Event,
    focus::{self, FocusAction},
    match_view,
    view::{
        HStack, Rotary, VStack, View,
        prelude::*,
        rotary::{RotaryEvent, RotaryState},
    },
};

use crate::{
    operation::OperationCommand,
    pin_digit,
    ui::{
        colour::{self, ColorFormat},
        font, keys, state,
    },
};

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
pub struct State {
    pin: [pin_digit::PinDigit; 4],
}

#[must_use]
pub fn view(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    VStack::new((
        Text::new("Enter PIN", &font::B612_REGULAR).foreground_color(colour::ON_BACKGROUND),
        Lens::new(pin_entry(&state.locked_state), |s: &mut state::State| {
            &mut s.locked_state
        }),
        Button::new(
            |state: &mut state::State| {
                if state.locked_state.pin
                    == state
                        .operation_state
                        .as_locked()
                        .and_then(|x| x.clone())
                        .unwrap_or_default()
                        .digits
                {
                    state.locked_state.pin = Default::default();
                    let _ = state.next_operation_commands.push(OperationCommand::Unlock);
                }
            },
            |bs| {
                Text::new("Confirm", &font::B612_REGULAR)
                    .padding(Edges::All, 4)
                    .foreground_color(if bs.is_focused() {
                        colour::ON_PRIMARY
                    } else {
                        colour::ON_PRIMARY_FIXED
                    })
                    .background_color(
                        if bs.is_focused() {
                            colour::PRIMARY
                        } else {
                            colour::PRIMARY_FIXED
                        },
                        RoundedRectangle::new(4),
                    )
            },
        ),
    ))
    .with_spacing(2)
    .with_alignment(HorizontalAlignment::Center)
    .flex_infinite_width(HorizontalAlignment::Center)
    .with_infinite_max_height()
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

fn pin_entry(state: &State) -> impl View<ColorFormat, State> + use<> {
    HStack::new((
        Lens::new(pin_piece(state.pin[0]), |s: &mut State| &mut s.pin[0]),
        Lens::new(pin_piece(state.pin[1]), |s: &mut State| &mut s.pin[1]),
        Lens::new(pin_piece(state.pin[2]), |s: &mut State| &mut s.pin[2]),
        Lens::new(pin_piece(state.pin[3]), |s: &mut State| &mut s.pin[3]),
    ))
}

fn pin_piece(pin: pin_digit::PinDigit) -> impl View<ColorFormat, pin_digit::PinDigit> {
    Rotary::new(
        |pin: &mut pin_digit::PinDigit, event: RotaryEvent| match event {
            RotaryEvent::Next => *pin = pin.prev(),
            RotaryEvent::Previous => *pin = pin.next(),
            RotaryEvent::Select | RotaryEvent::Exit => {}
        },
        move |rotary_state| {
            Text::new(pin.as_str(), &font::B612_REGULAR_LARGE_NUMBERS)
            .padding(Edges::All, 4)
            .foreground_color(
                match rotary_state {
                    RotaryState::UnFocused => colour::ON_BACKGROUND,
                    RotaryState::Focused => colour::ON_BACKGROUND,
                    RotaryState::Captive => colour::ON_PRIMARY_FIXED,
                }
            )
            .background(Alignment::Center,
                        match_view!(rotary_state, {
                            RotaryState::UnFocused => EmptyView,
                            RotaryState::Focused => RoundedRectangle::new(4).stroked(2).foreground_color(colour::PRIMARY),
                            RotaryState::Captive => RoundedRectangle::new(4).stroked(2).foreground_color(colour::PRIMARY_FIXED)
                        })
            )
                .content_shape(Rectangle.corner_radius(4))
        },
    )
}
