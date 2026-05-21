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

use crate::{
    operation::OperationCommand,
    ui::{
        colour::{self, ColorFormat},
        font, keys, state,
    },
};

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
#[repr(u8)]
pub enum PinDigit {
    #[default]
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
}

impl PinDigit {
    pub fn as_char(self) -> char {
        char::from_digit(self as u32, 10).unwrap()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PinDigit::D0 => "0",
            PinDigit::D1 => "1",
            PinDigit::D2 => "2",
            PinDigit::D3 => "3",
            PinDigit::D4 => "4",
            PinDigit::D5 => "5",
            PinDigit::D6 => "6",
            PinDigit::D7 => "7",
            PinDigit::D8 => "8",
            PinDigit::D9 => "9",
        }
    }

    pub fn next(self) -> Self {
        use PinDigit::*;

        match self {
            D0 => D1,
            D1 => D2,
            D2 => D3,
            D3 => D4,
            D4 => D5,
            D5 => D6,
            D6 => D7,
            D7 => D8,
            D8 => D9,
            D9 => D0,
        }
    }

    pub fn prev(self) -> Self {
        use PinDigit::*;

        match self {
            D0 => D9,
            D1 => D0,
            D2 => D1,
            D3 => D2,
            D4 => D3,
            D5 => D4,
            D6 => D5,
            D7 => D6,
            D8 => D7,
            D9 => D8,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
pub struct State {
    pin: [PinDigit; 4],
}

#[must_use]
pub fn view(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    VStack::new((
        Text::new("Enter PIN", &font::B612_REGULAR)
            .multiline_text_alignment(HorizontalTextAlignment::Center)
            .foreground_color(colour::ON_BACKGROUND),
        Lens::new(pin_entry(&state.locked_state), |s: &mut state::State| {
            &mut s.locked_state
        }),
        Button::new(
            |state: &mut state::State| {
                // TODO: make this changeable. I'll do this when there's more
                // than one user :)
                if state.locked_state.pin
                    == [PinDigit::D0, PinDigit::D0, PinDigit::D0, PinDigit::D0]
                    // == [PinDigit::D2, PinDigit::D7, PinDigit::D0, PinDigit::D8]
                {
                    state.locked_state.pin = Default::default();
                    state.next_operation_command = Some(OperationCommand::Unlock);
                }
            },
            |bs| {
                Text::new("Confirm", &font::B612_REGULAR)
                    .multiline_text_alignment(HorizontalTextAlignment::Center)
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

fn pin_entry(state: &State) -> impl View<ColorFormat, State> + use<> {
    HStack::new((
        Lens::new(pin_piece(state.pin[0]), |s: &mut State| &mut s.pin[0]),
        Lens::new(pin_piece(state.pin[1]), |s: &mut State| &mut s.pin[1]),
        Lens::new(pin_piece(state.pin[2]), |s: &mut State| &mut s.pin[2]),
        Lens::new(pin_piece(state.pin[3]), |s: &mut State| &mut s.pin[3]),
    ))
}

fn pin_piece(pin: PinDigit) -> impl View<ColorFormat, PinDigit> {
    Rotary::new(
        |pin: &mut PinDigit, event: RotaryEvent| match event {
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
