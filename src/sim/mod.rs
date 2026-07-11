use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    adc::{AmbientLight, Throttle},
    buttons_proto::Buttons,
    cfg::{HeadlightMode, SpeedLimit, SpeedMode, UnlockCode},
    operation::{ActiveState, HeadlightConfig, OperationCommand, OperationState},
    system_state::{BatteryDebug, BatteryInfo, SystemState, SystemVoltage},
    ui,
};

static CAN_ALIVE: AtomicBool = AtomicBool::new(true);

pub fn can_is_alive() -> bool {
    CAN_ALIVE.load(Ordering::Relaxed)
}

pub fn set_can_alive(alive: bool) {
    CAN_ALIVE.store(alive, Ordering::Relaxed);
}

pub fn default_system_state() -> SystemState {
    SystemState {
        motor_speed: 0,
        headlight_on: false,
        brake_light_on: false,
        controller_temp: 25,
        system_voltage: SystemVoltage {
            from_controller: 5400,
            from_battery: 4200,
        },
        controller_speed_limit_mode: false,
        battery_current: 0,
        battery_debug: BatteryDebug {
            command: 0,
            state: 0,
            estimated_range: 0,
        },
        battery_info: BatteryInfo {
            level_from_controller: 80,
            relative_soc: 80,
            absolute_soc: 4000,
            relative_soh: 95,
            absolute_soh: 4800,
            capacity: 5000,
            charged: false,
            charging: false,
            temperature: 25,
        },
        throttle: Throttle::INITIAL,
        ambient_light: AmbientLight::INITIAL,
        buttons: Buttons::empty(),
        odometer: 123,
        predicted_range: 25,
    }
}

pub fn default_operation_state() -> OperationState {
    OperationState::Locked(Some(UnlockCode::default()))
}

pub struct SimState {
    pub system: SystemState,
    pub operation: OperationState,
    unlock_code: UnlockCode,
    speed_limit: SpeedLimit,
    speed_mode: SpeedMode,
    headlight_mode: HeadlightMode,
}

impl SimState {
    pub fn new() -> Self {
        Self {
            system: default_system_state(),
            operation: default_operation_state(),
            unlock_code: UnlockCode::default(),
            speed_limit: SpeedLimit::default(),
            speed_mode: SpeedMode::default(),
            headlight_mode: HeadlightMode::default(),
        }
    }

    pub fn needs_ui_sync(&self, ui: &ui::state::State) -> bool {
        ui.system_state != self.system || ui.operation_state != self.operation
    }

    pub fn sync_to_ui(&self, state: &mut ui::state::State) {
        if state.system_state != self.system {
            state.system_state = self.system.clone();
        }
        if state.operation_state != self.operation {
            state.operation_state = self.operation.clone();
        }
    }

    pub fn apply_operation_commands(
        &mut self,
        commands: impl IntoIterator<Item = OperationCommand>,
    ) {
        for cmd in commands {
            match cmd {
                OperationCommand::Unlock => {
                    self.operation = OperationState::Active(ActiveState {
                        throttle: self.system.throttle,
                        speed_limit: self.speed_limit.get_validated(),
                        speed_limit_unlocked: false,
                        walk_mode_counter: None,
                        speed_mode: self.speed_mode,
                        headlight_mode: self.headlight_mode,
                        headlight_config: HeadlightConfig {
                            low: 10,
                            high: 30,
                            auto_on: false,
                        },
                    });
                }
                OperationCommand::Lock => {
                    self.operation = OperationState::Locked(Some(self.unlock_code));
                }
                OperationCommand::SetSpeedLimit(new_limit) => {
                    self.speed_limit = SpeedLimit::new_validated(new_limit);
                    if let OperationState::Active(ref mut active) = self.operation {
                        active.speed_limit = self.speed_limit.get_validated();
                    }
                }
                OperationCommand::SetSpeedMode(speed_mode) => {
                    self.speed_mode = speed_mode;
                    if let OperationState::Active(ref mut active) = self.operation {
                        active.speed_mode = speed_mode;
                    }
                }
                OperationCommand::SetHeadlightMode(headlight_mode) => {
                    self.headlight_mode = headlight_mode;
                    if let OperationState::Active(ref mut active) = self.operation {
                        active.headlight_mode = headlight_mode;
                    }
                }
                OperationCommand::UnlockSpeedLimit => {
                    if let OperationState::Active(ref mut active) = self.operation {
                        active.speed_limit_unlocked = true;
                    }
                }
                OperationCommand::LockSpeedLimit => {
                    if let OperationState::Active(ref mut active) = self.operation {
                        active.speed_limit_unlocked = false;
                    }
                }
            }
        }
    }

    pub fn set_locked(&mut self, locked: bool) {
        if locked {
            if self.operation.is_locked() {
                return;
            }
            self.operation = OperationState::Locked(Some(self.unlock_code));
        } else if self.operation.is_locked() {
            self.apply_operation_commands([OperationCommand::Unlock]);
        }
    }

    pub fn set_unlock_code_digits(&mut self, digits: [crate::pin_digit::PinDigit; 4]) {
        if self.unlock_code.digits == digits {
            return;
        }
        self.unlock_code.digits = digits;
        if let OperationState::Locked(code) = &mut self.operation {
            *code = Some(self.unlock_code);
        }
    }
}
