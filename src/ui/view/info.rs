use buoyant::{
    event::Event,
    focus::{self, FocusAction},
    view::{View, prelude::*, scroll_view::ScrollBarVisibility},
};
use strum::{EnumCount as _, VariantArray};

use crate::{
    system_state,
    ui::{
        colour::{self, ColorFormat},
        font, keys,
        state::{self},
    },
};

#[must_use]
pub fn view(_state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    ScrollView::new(
        ForEach::<{ Info::COUNT }>::new_vertical(Info::VARIANTS, move |s| info_entry(*s))
            .with_spacing(8),
    )
    .with_bar_visibility(ScrollBarVisibility::Never)
    .padding(Edges::All, 8)
    .captures_event(|e, s: &mut state::State| match e {
        Event::KeyDown(keys::UP_CLICK) => Some(FocusAction::Previous.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::DOWN_CLICK) => Some(FocusAction::Next.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::CONFIRM_HOLD) => {
            s.page_action = Some(state::PageAction::ExitSettings);
            None
        }
        _ => Some(e.clone()),
    })
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, strum::EnumCount, strum::VariantArray)]
pub enum Info {
    SystemVoltageController = 0,
    SystemVoltageBattery,
    BatteryCurrent,
    BatteryCommand,
    BatteryState,
    BatteryRange,
    BatteryRelSOC,
    BatteryAbsSOC,
    BatteryRelSOH,
    BatteryAbsSOH,
    BatteryCapacity,
    BatteryCharging,
    BatteryCharged,
    BatteryTemperature,
}

impl Info {
    fn name(self) -> &'static str {
        match self {
            Info::SystemVoltageController => "Voltage (ctrl)",
            Info::SystemVoltageBattery => "Voltage (bat)",
            Info::BatteryCurrent => "Current",
            Info::BatteryCommand => "Bat Cmd",
            Info::BatteryState => "Bat State",
            Info::BatteryRange => "Bat Range",
            Info::BatteryRelSOC => "Rel SoC",
            Info::BatteryAbsSOC => "Abs SoC",
            Info::BatteryRelSOH => "Rel SoH",
            Info::BatteryAbsSOH => "Abs SoH",
            Info::BatteryCapacity => "Capacity",
            Info::BatteryCharging => "Charging",
            Info::BatteryCharged => "Charged",
            Info::BatteryTemperature => "Bat temp",
        }
    }

    fn val(self) -> heapless::String<8, u8> {
        let mut s = heapless::String::new();

        let _ = system_state::read_state(|st| match self {
            Info::SystemVoltageController => {
                ufmt::uwrite!(&mut s, "{}", st.system_voltage.from_controller)
            }
            Info::SystemVoltageBattery => {
                ufmt::uwrite!(&mut s, "{}", st.system_voltage.from_battery)
            }
            Info::BatteryCurrent => ufmt::uwrite!(&mut s, "{}", st.battery_current),
            Info::BatteryCommand => ufmt::uwrite!(&mut s, "{}", st.battery_debug.command),
            Info::BatteryState => ufmt::uwrite!(&mut s, "{}", st.battery_debug.state),
            Info::BatteryRange => ufmt::uwrite!(&mut s, "{}", st.battery_debug.estimated_range),
            Info::BatteryRelSOC => ufmt::uwrite!(&mut s, "{}", st.battery_info.relative_soc),
            Info::BatteryAbsSOC => ufmt::uwrite!(&mut s, "{}", st.battery_info.absolute_soc),
            Info::BatteryRelSOH => ufmt::uwrite!(&mut s, "{}", st.battery_info.relative_soh),
            Info::BatteryAbsSOH => ufmt::uwrite!(&mut s, "{}", st.battery_info.absolute_soh),
            Info::BatteryCapacity => ufmt::uwrite!(&mut s, "{}", st.battery_info.capacity),
            Info::BatteryCharging => ufmt::uwrite!(&mut s, "{}", st.battery_info.charging),
            Info::BatteryCharged => ufmt::uwrite!(&mut s, "{}", st.battery_info.charged),
            Info::BatteryTemperature => {
                ufmt::uwrite!(&mut s, "{}", st.battery_info.temperature)
            }
        });

        s
    }
}

fn info_entry(info: Info) -> impl View<ColorFormat, state::State> + use<> {
    Button::new(
        move |_s: &mut state::State| {},
        move |bs| {
            let (fg, bg) = if bs.is_focused() {
                (colour::ON_TERTIARY_CONTAINER, colour::TERTIARY_CONTAINER)
            } else {
                (colour::ON_SECONDARY_CONTAINER, colour::SECONDARY_CONTAINER)
            };

            HStack::new((
                Text::new(info.name(), &font::B612_REGULAR).foreground_color(fg),
                Text::new(info.val(), &font::B612_SMALL)
                    .foreground_color(fg)
                    .flex_infinite_width(HorizontalAlignment::Trailing),
            ))
            .with_alignment(VerticalAlignment::Center)
            .with_spacing(4)
            .padding(Edges::All, 8)
            .flex_infinite_width(HorizontalAlignment::Leading)
            .background_color(bg, RoundedRectangle::new(8))
            .padding(Edges::Horizontal, 8)
        },
    )
}
