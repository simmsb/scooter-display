use buoyant::{
    event::Event,
    focus::{self, FocusAction},
    if_view,
    view::{VStack, View, prelude::*, scroll_view::ScrollBarVisibility},
};
use strum::{EnumCount as _, VariantArray};

use crate::{
    operation::{self, OperationCommand},
    ui::{
        colour::{self, ColorFormat},
        font, keys,
        state::{self, PageAction},
    },
};

#[must_use]
pub fn view(state: &state::State) -> impl View<ColorFormat, state::State> + use<> {
    let no_speeding = state.no_speeding;
    let open_menu = state.settings_state.open_menu;

    ScrollView::new(
        ForEach::<{ Setting::COUNT }>::new_vertical(Setting::VARIANTS, move |s| {
            setting_entry(no_speeding, *s)
        })
        .with_spacing(8),
    )
    .with_bar_visibility(ScrollBarVisibility::Never)
    .popover(open_menu.as_ref(), |setting| {
        VStack::new((
            Text::new(setting.name(), &font::B612_REGULAR)
                .foreground_color(colour::ON_SECONDARY_CONTAINER),
            Spacer::default(),
            ForEach::<{ max_entries_of_setting() }>::new_vertical(setting.entries(), |e| {
                Button::new(
                    |s: &mut state::State| (e.cb)(s),
                    |bs| {
                        let (fg, bg) = if bs.is_focused() {
                            (colour::ON_TERTIARY_CONTAINER, colour::TERTIARY_CONTAINER)
                        } else {
                            (colour::ON_SECONDARY_CONTAINER, colour::SECONDARY_CONTAINER)
                        };

                        Text::new(e.name, &font::B612_SMALL)
                            .foreground_color(fg)
                            .flex_infinite_width(HorizontalAlignment::Leading)
                            .padding(Edges::All, 8)
                            .background_color(bg, RoundedRectangle::new(4))
                    },
                )
            }),
            Spacer::default(),
            Button::new(
                |s: &mut state::State| {
                    s.settings_state.open_menu = None;
                },
                |bs| {
                    let (fg, bg) = if bs.is_focused() {
                        (colour::ON_TERTIARY_CONTAINER, colour::TERTIARY_CONTAINER)
                    } else {
                        (colour::ON_SECONDARY_CONTAINER, colour::SECONDARY_CONTAINER)
                    };

                    Text::new("Back", &font::B612_SMALL)
                        .foreground_color(fg)
                        .flex_infinite_width(HorizontalAlignment::Leading)
                        .padding(Edges::All, 8)
                        .background_color(bg, RoundedRectangle::new(4))
                },
            ),
        ))
        .flex_frame()
        .with_alignment(Alignment::Center)
        .with_infinite_max_dimensions()
        .padding(Edges::All, 8)
        .background_color(colour::SECONDARY_CONTAINER, RoundedRectangle::new(8))
    })
    .padding(Edges::All, 8)
    .captures_event(|e, s: &mut state::State| match e {
        Event::KeyDown(keys::UP_CLICK) => Some(FocusAction::Previous.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::DOWN_CLICK) => Some(FocusAction::Next.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::CONFIRM_CLICK) => Some(FocusAction::Select.into_event(focus::GROUP_0)),
        Event::KeyDown(keys::CONFIRM_HOLD) => {
            s.page_action = Some(state::PageAction::ExitSettings);
            s.settings_state.open_menu = None;
            None
        }
        _ => Some(e.clone()),
    })
}

struct SettingEntry {
    name: &'static str,
    cb: &'static dyn Fn(&mut state::State),
}

impl SettingEntry {
    const fn new(name: &'static str, cb: &'static impl Fn(&mut state::State)) -> Self {
        Self { name, cb }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, strum::EnumCount, strum::VariantArray)]
pub enum Setting {
    Language,
    SpeedLimit,
    Info,
}

const fn max_entries_of_setting() -> usize {
    let mut i = 0;
    let settings = Setting::VARIANTS;
    let mut max = 0;

    while i < settings.len() {
        let e = settings[i].entries().len();

        if e > max {
            max = e;
        }

        i += 1;
    }

    max
}

impl Setting {
    fn is_guarded(self) -> bool {
        match self {
            Self::SpeedLimit => true,
            _ => false,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Setting::Language => "Language",
            Setting::SpeedLimit => "Speed limit",
            Setting::Info => "Info",
        }
    }

    fn desc(self) -> &'static str {
        match self {
            Setting::Language => "Set the system language",
            Setting::SpeedLimit => "Set the speed limit",
            Setting::Info => "View info and statistics",
        }
    }

    const fn entries(self) -> &'static [SettingEntry] {
        match self {
            Setting::Language => const { &[SettingEntry::new("English", &|_| {})] },
            Setting::SpeedLimit => {
                const {
                    &[
                        SettingEntry::new("22", &|s| {
                            s.next_operation_command = Some(OperationCommand::SetSpeedLimit(22));
                        }),
                        SettingEntry::new("25", &|s| {
                            s.next_operation_command = Some(OperationCommand::SetSpeedLimit(25));
                        }),
                        SettingEntry::new("35", &|s| {
                            s.next_operation_command = Some(OperationCommand::SetSpeedLimit(35));
                        }),
                        SettingEntry::new("45", &|s| {
                            s.next_operation_command = Some(OperationCommand::SetSpeedLimit(45));
                        }),
                    ]
                }
            }
            Setting::Info => const { &[] },
        }
    }

    const fn triggers_page(self) -> Option<PageAction> {
        match self {
            Setting::Info => Some(PageAction::EnterInfo),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
pub struct State {
    open_menu: Option<Setting>,
}

fn setting_entry(
    no_speeding: bool,
    setting: Setting,
) -> impl View<ColorFormat, state::State> + use<> {
    if_view!((setting.is_guarded() && no_speeding) {
        EmptyView
    } else {
        Button::new(
            move |s: &mut state::State| {
                if let Some(next_page) = setting.triggers_page() {
                    s.page_action = Some(next_page);
                } else {
                    s.settings_state.open_menu = Some(setting);
                }
            },
            move |bs| {
                let (fg, bg) = if bs.is_focused() {
                    (colour::ON_TERTIARY_CONTAINER, colour::TERTIARY_CONTAINER)
                } else {
                    (colour::ON_SECONDARY_CONTAINER, colour::SECONDARY_CONTAINER)
                };

                VStack::new((
                    Text::new(setting.name(), &font::B612_REGULAR).foreground_color(fg) ,
                    Text::new(setting.desc(), &font::B612_SMALL).foreground_color(fg),
                ))
                .with_alignment(HorizontalAlignment::Leading)
                .with_spacing(4)
                .padding(Edges::All, 8)
                .flex_infinite_width(HorizontalAlignment::Leading)
                .background_color(bg, RoundedRectangle::new(8))
                .padding(Edges::Horizontal, 8)
            },
        )
    })
}
