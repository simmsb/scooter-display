use crate::{
    can_proto::SpeedMode,
    operation::{OperationCommand, OperationState},
    system_state::SystemState,
};

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format, Default)]
pub enum Page {
    #[default]
    Home,
    Settings,
}

impl Page {
    pub fn handle_action(&self, action: PageAction) -> Option<Self> {
        match (self, action) {
            (Page::Home, PageAction::EnterSettings) => Some(Page::Settings),
            (Page::Home, _) => return None,
            (Page::Settings, PageAction::ExitSettings) => Some(Page::Home),
            (Page::Settings, _) => return None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PageAction {
    EnterSettings,
    ExitSettings,
}

pub struct State {
    pub page: Page,

    pub locked_state: super::view::locked::State,

    pub system_state: SystemState,
    pub operation_state: OperationState,
    pub page_action: Option<PageAction>,
    pub next_operation_command: Option<OperationCommand>,
}

impl State {
    pub fn new() -> Self {
        Self {
            page: Default::default(),
            locked_state: Default::default(),
            system_state: crate::system_state::read_state(|s| s.clone()),
            operation_state: crate::operation::read_state(|s| s.clone()),
            page_action: None,
            next_operation_command: None,
        }
    }
}
