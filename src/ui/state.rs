use chrono::Timelike as _;

use crate::{
    operation::{OperationCommand, OperationState},
    system_state::SystemState,
};

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
pub enum Page {
    #[default]
    Home,
    Settings,
    Info,
}

impl Page {
    pub fn handle_action(&self, action: PageAction) -> Option<Self> {
        // This is a bit redundant, but we might want to case it on the
        // current page in the future
        match (self, action) {
            (_, PageAction::EnterSettings) => Some(Page::Settings),
            (_, PageAction::ExitSettings) => Some(Page::Home),
            (_, PageAction::EnterInfo) => Some(Page::Info),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PageAction {
    EnterSettings,
    ExitSettings,
    EnterInfo,
}

pub struct State {
    pub page: Page,

    pub locked_state: super::view::locked::State,
    pub home_state: super::view::home::State,
    pub settings_state: super::view::settings::State,

    pub hour: u8,
    pub minute: u8,
    pub second: u8,

    pub system_state: SystemState,
    pub operation_state: OperationState,
    pub page_action: Option<PageAction>,
    pub next_operation_commands: heapless::Vec<OperationCommand, 3, u8>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        let now = crate::platform::current_time();

        Self {
            page: Default::default(),
            locked_state: Default::default(),
            home_state: Default::default(),
            settings_state: Default::default(),
            hour: now.hour() as u8,
            minute: now.minute() as u8,
            second: now.second() as u8,
            #[cfg(feature = "sim")]
            system_state: crate::sim::default_system_state(),
            #[cfg(not(feature = "sim"))]
            system_state: crate::system_state::read_state(|s| s.clone()),
            #[cfg(feature = "sim")]
            operation_state: crate::sim::default_operation_state(),
            #[cfg(not(feature = "sim"))]
            operation_state: crate::operation::read_state(|s| s.clone()),
            page_action: None,
            next_operation_commands: Default::default(),
        }
    }
}
