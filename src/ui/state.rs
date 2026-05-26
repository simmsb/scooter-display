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
        Self {
            page: Default::default(),
            locked_state: Default::default(),
            home_state: Default::default(),
            settings_state: Default::default(),
            system_state: crate::system_state::read_state(|s| s.clone()),
            operation_state: crate::operation::read_state(|s| s.clone()),
            page_action: None,
            next_operation_commands: Default::default(),
        }
    }
}
