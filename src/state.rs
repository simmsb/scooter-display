use crate::can_proto::*;

static STATE_UPDATES: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    (),
    4,
> = embassy_sync::watch::Watch::new_with(());

static STATE: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    SystemState,
> = embassy_sync::blocking_mutex::Mutex::new(SystemState::DEFAULT);

pub fn read_state<T>(f: impl for<'a> FnOnce(&'a SystemState) -> T) -> T {
    STATE.lock(f)
}

fn update_state(f: impl for<'a> FnOnce(&'a mut SystemState)) {
    unsafe {
        STATE.lock_mut(f);
    }
}

pub static CAN_MESSAGES: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    CanMessage,
    1,
> = embassy_sync::channel::Channel::new();

pub static BT_COMMANDS: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    (),
    1,
> = embassy_sync::channel::Channel::new();

#[derive(PartialEq, Eq, defmt::Format, Default)]
pub struct BatteryLevel {
    from_controller: u8,
    from_battery: u8,
}

#[derive(PartialEq, Eq, defmt::Format, Default)]
pub struct SystemVoltage {
    from_controller: u16,
    from_battery: u32,
}

#[derive(PartialEq, Eq, defmt::Format, Default)]
pub struct BatteryDebug {
    command: u16,
    state: u16,
    estimated_range: u32,
}

#[derive(PartialEq, Eq, defmt::Format, Default)]
pub struct BatteryInfo {
    relative_soc: u32,
    absolute_soc: u32,
    relative_soh: u8,
    absolute_soh: u32,
    capacity: u16,
    charged: bool,
    temperature: i16,
}

#[derive(PartialEq, Eq, defmt::Format, Default)]
pub struct BatteryChargeEntry {
    when: chrono::DateTime<chrono::Utc>,
    charge: u16,
}

#[derive(PartialEq, Eq, defmt::Format, Default)]
pub struct SystemState {
    pub motor_speed: u16,
    pub headlight_on: bool,
    pub brake_light_on: bool,

    pub controller_temp: u8,
    pub system_voltage: SystemVoltage,
    pub controller_speed_limit_mode: bool,

    pub battery_current: i32,
    pub battery_level: BatteryLevel,
    pub battery_debug: BatteryDebug,
    pub battery_info: BatteryInfo,

    pub charges: [Option<BatteryChargeEntry>; 16],
}

impl SystemState {
    const DEFAULT: Self = SystemState {
        motor_speed: 0,
        headlight_on: false,
        brake_light_on: false,
        controller_temp: 0,
        system_voltage: SystemVoltage {
            from_controller: 0,
            from_battery: 0,
        },
        controller_speed_limit_mode: false,
        battery_current: 0,
        battery_level: BatteryLevel {
            from_controller: 0,
            from_battery: 0,
        },
        battery_debug: BatteryDebug {
            command: 0,
            state: 0,
            estimated_range: 0,
        },
        battery_info: BatteryInfo {
            relative_soc: 0,
            absolute_soc: 0,
            relative_soh: 0,
            absolute_soh: 0,
            capacity: 0,
            charged: false,
            temperature: 0,
        },
        charges: [const { None }; _],
    };
}

#[embassy_executor::task]
pub async fn system_state_updater() {
    system_state_updater_().await
}

async fn system_state_updater_() {
    let can_messages = CAN_MESSAGES.receiver();
    let bt_commands = BT_COMMANDS.receiver();
    let state_updated = STATE_UPDATES.sender();

    loop {
        match embassy_futures::select::select(can_messages.receive(), bt_commands.receive()).await {
            embassy_futures::select::Either::First(can_msg) => {
                update_state(|s| s.update_from_can_message(&can_msg));
            }
            embassy_futures::select::Either::Second(_) => {}
        }

        state_updated.send(());
    }
}

impl SystemState {
    pub fn update_from_can_message(&mut self, msg: &CanMessage) {
        match msg {
            CanMessage::ControllerStatus(ControllerStatus { battery_level, .. }) => {
                self.battery_level.from_controller = *battery_level;
            }
            CanMessage::ControllerSpeed(ControllerSpeed {
                motor_speed,
                headlight_on,
                brake_light_on,
                ..
            }) => {
                self.motor_speed = *motor_speed;
                self.headlight_on = *headlight_on;
                self.brake_light_on = *brake_light_on;
            }
            CanMessage::ControllerTempMotor(ControllerTempMotor { temp, voltage }) => {
                self.controller_temp = *temp;
                self.system_voltage.from_controller = *voltage;
            }
            CanMessage::ControllerSpeedMode(ControllerSpeedMode { .. }) => {}
            CanMessage::ControllerSpeedLimit(ControllerSpeedLimit { speed_limit }) => {
                self.controller_speed_limit_mode = *speed_limit;
            }
            CanMessage::BatteryCommandState(BatteryCommandState {
                command,
                state,
                estimated_range,
            }) => {
                self.battery_debug = BatteryDebug {
                    command: *command,
                    state: *state,
                    estimated_range: *estimated_range,
                }
            }
            CanMessage::BatteryVoltageCurrent(BatteryVoltageCurrent {
                voltage_mv,
                current_ma,
            }) => {
                self.system_voltage.from_battery = *voltage_mv;
                self.battery_current = *current_ma;
            }
            CanMessage::BatteryChargeLevel(BatteryChargeLevel {
                relative_soc,
                absolute_soc_mah,
            }) => {
                self.battery_info.relative_soc = *relative_soc;
                self.battery_info.absolute_soc = *absolute_soc_mah;
            }
            CanMessage::BatteryStateOfHealth(BatteryStateOfHealth {
                relative_soh,
                absolute_soh_mah,
            }) => {
                self.battery_info.relative_soh = *relative_soh;
                self.battery_info.absolute_soh = *absolute_soh_mah;
            }
            CanMessage::BatteryCapacityTemp(BatteryCapacityTemp {
                capacity_mah,
                battery_charged,
                battery_temp,
            }) => {
                self.battery_info.capacity = *capacity_mah;
                self.battery_info.charged = *battery_charged;
                self.battery_info.temperature = *battery_temp;
            }
            CanMessage::BatteryChargeHistoryEntry(BatteryChargeHistoryEntry {
                idx,
                year,
                month,
                day,
                hour,
                minute,
                second,
                ..
            }) => {
                let Some(d) = chrono::NaiveDate::from_ymd_opt(
                    *year as i32 + 2000,
                    *month as u32,
                    *day as u32,
                ) else {
                    return;
                };
                let Some(t) =
                    chrono::NaiveTime::from_hms_opt(*hour as u32, *minute as u32, *second as u32)
                else {
                    return;
                };
                let dt = chrono::NaiveDateTime::new(d, t);

                let Some(entry) = self.charges.get_mut(*idx as usize) else {
                    return;
                };
                entry.get_or_insert_default().when = dt.and_utc();
            }
            CanMessage::BatteryChargeHistoryCharge(BatteryChargeHistoryCharge {
                idx,
                charge,
                ..
            }) => {
                let Some(entry) = self.charges.get_mut(*idx as usize) else {
                    return;
                };
                entry.get_or_insert_default().charge = *charge;
            }
        }
    }
}
