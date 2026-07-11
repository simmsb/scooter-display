#[cfg(feature = "app")]
use embassy_futures::select;
#[cfg(feature = "app")]
use embassy_time::{Duration, Ticker};
#[cfg(feature = "app")]
use no_std_moving_average::MovingAverage;

use crate::{
    adc::{AmbientLight, Throttle},
    buttons_proto::Buttons,
    can_proto::*,
};

#[cfg(feature = "app")]
use crate::{
    averager::Averager,
    buttons::BUTTON_STATE_WATCH,
    cfg::{Odometer, Storable},
};

pub static STATE_UPDATES: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    (),
    4,
> = embassy_sync::watch::Watch::new_with(());

static STATE: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    SystemState,
> = embassy_sync::blocking_mutex::Mutex::new(SystemState::DEFAULT);

pub fn read_state<T>(f: impl for<'a> FnOnce(&'a SystemState) -> T) -> T {
    STATE.lock(f)
}

fn update_state<T>(f: impl for<'a> FnOnce(&'a mut SystemState) -> T) -> T {
    unsafe { STATE.lock_mut(f) }
}

pub static CAN_MESSAGES: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    CanMessage,
    1,
> = embassy_sync::channel::Channel::new();

pub static BT_COMMANDS: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    (),
    1,
> = embassy_sync::channel::Channel::new();

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct SystemVoltage {
    pub from_controller: u16,
    pub from_battery: u16,
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct BatteryDebug {
    pub command: u16,
    pub state: u16,
    pub estimated_range: u16,
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct BatteryInfo {
    pub level_from_controller: u8,
    pub relative_soc: u8,
    pub absolute_soc: u16,
    pub relative_soh: u8,
    pub absolute_soh: u16,
    pub capacity: u16,
    pub charged: bool,
    pub charging: bool,
    pub temperature: i16,
}

#[derive(PartialEq, Eq, defmt::Format, Default, Clone)]
pub struct BatteryChargeEntry {
    pub when: chrono::DateTime<chrono::Utc>,
    pub charge: u16,
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct SystemState {
    /// motor speed, in deca meters per hour (speed / 100 = km/h)
    pub motor_speed: u16,
    pub headlight_on: bool,
    pub brake_light_on: bool,

    pub controller_temp: u8,
    pub system_voltage: SystemVoltage,
    pub controller_speed_limit_mode: bool,

    pub battery_current: i16,
    pub battery_debug: BatteryDebug,
    pub battery_info: BatteryInfo,

    pub throttle: Throttle,
    pub ambient_light: AmbientLight,

    pub buttons: Buttons,
    // pub charges: [Option<BatteryChargeEntry>; 16],
    /// in km
    pub odometer: u16,

    /// in km
    pub predicted_range: u16,
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
        battery_debug: BatteryDebug {
            command: 0,
            state: 0,
            estimated_range: 0,
        },
        battery_info: BatteryInfo {
            level_from_controller: 0,
            relative_soc: 0,
            absolute_soc: 0,
            relative_soh: 0,
            absolute_soh: 0,
            capacity: 0,
            charged: false,
            charging: false,
            temperature: 0,
        },
        throttle: Throttle::INITIAL,
        ambient_light: AmbientLight::INITIAL,
        buttons: Buttons::empty(),
        odometer: 0,
        predicted_range: 0,
        // charges: [const { None }; _],
    };

    fn update_from_adc_reading(&mut self, reading: crate::adc::AdcReading) -> bool {
        match reading {
            crate::adc::AdcReading::Throttle(throttle) => {
                if self.throttle != throttle {
                    self.throttle = throttle;
                    return true;
                }
            }
            crate::adc::AdcReading::AmbientLight(ambient_light) => {
                if self.ambient_light != ambient_light {
                    self.ambient_light = ambient_light;
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(feature = "app")]
#[derive(Default)]
struct PrivateState {
    average_speed: Averager,
    average_current: Averager,
    current_history: MovingAverage<u16, u32, 8>,
    speed_history: MovingAverage<u16, u32, 8>,
    moving_average_speed: u16,
    predicted_range: u16,
}

#[cfg(feature = "app")]
#[embassy_executor::task]
pub async fn system_state_updater() {
    system_state_updater_().await
}

#[cfg(feature = "app")]
async fn system_state_updater_() {
    let can_messages = CAN_MESSAGES.receiver();
    let bt_commands = BT_COMMANDS.receiver();
    let mut adc_readings = crate::adc::ADC_READINGS.subscriber().unwrap();
    let state_updated = STATE_UPDATES.sender();
    let mut buttons_reader = BUTTON_STATE_WATCH.receiver().unwrap();

    let mut update_private_state_ticker = Ticker::every(Duration::from_secs(30));

    let mut private_state = PrivateState::default();

    loop {
        let updated = match select::select5(
            can_messages.receive(),
            bt_commands.receive(),
            adc_readings.next_message_pure(),
            buttons_reader.changed(),
            update_private_state_ticker.next(),
        )
        .await
        {
            select::Either5::First(can_msg) => {
                update_state(|s| s.update_from_can_message(&can_msg));
                private_state.update_from_can_message(&can_msg);
                true
            }
            select::Either5::Second(_) => false,
            select::Either5::Third(reading) => update_state(|s| s.update_from_adc_reading(reading)),
            select::Either5::Fourth(buttons) => {
                update_state(|s| s.buttons = buttons);
                true
            }
            select::Either5::Fifth(_) => {
                private_state.periodic_update();
                update_state(|s| private_state.update_public(s));
                true
            }
        };

        if updated {
            state_updated.send(());
        }
    }
}

impl SystemState {
    pub fn update_from_can_message(&mut self, msg: &CanMessage) {
        match msg {
            CanMessage::ControllerStatus(ControllerStatus { battery_level, .. }) => {
                self.battery_info.level_from_controller = *battery_level;
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
                    estimated_range: estimated_range.truncate(),
                }
            }
            CanMessage::BatteryVoltageCurrent(BatteryVoltageCurrent {
                voltage_mv,
                current_ma,
            }) => {
                self.system_voltage.from_battery = voltage_mv.truncate();
                self.battery_current = current_ma.truncate();
            }
            CanMessage::BatteryChargeLevel(BatteryChargeLevel {
                relative_soc,
                absolute_soc_mah,
            }) => {
                self.battery_info.relative_soc = relative_soc.truncate();
                self.battery_info.absolute_soc = absolute_soc_mah.truncate();
            }
            CanMessage::BatteryStateOfHealth(BatteryStateOfHealth {
                relative_soh,
                absolute_soh_mah,
            }) => {
                self.battery_info.relative_soh = *relative_soh;
                self.battery_info.absolute_soh = absolute_soh_mah.truncate();
            }
            CanMessage::BatteryCapacityTemp(BatteryCapacityTemp {
                capacity_mah,
                battery_charged,
                battery_charging,
                battery_temp,
            }) => {
                self.battery_info.capacity = *capacity_mah;
                self.battery_info.charged = *battery_charged;
                self.battery_info.charging = *battery_charging;
                self.battery_info.temperature = *battery_temp;
            }
            _ => {} // CanMessage::BatteryChargeHistoryEntry(BatteryChargeHistoryEntry {
                    //     idx,
                    //     year,
                    //     month,
                    //     day,
                    //     hour,
                    //     minute,
                    //     second,
                    //     ..
                    // }) => {
                    //     let Some(d) = chrono::NaiveDate::from_ymd_opt(
                    //         *year as i32 + 2000,
                    //         *month as u32,
                    //         *day as u32,
                    //     ) else {
                    //         return;
                    //     };
                    //     let Some(t) =
                    //         chrono::NaiveTime::from_hms_opt(*hour as u32, *minute as u32, *second as u32)
                    //     else {
                    //         return;
                    //     };
                    //     let dt = chrono::NaiveDateTime::new(d, t);

                    //     let Some(entry) = self.charges.get_mut(*idx as usize) else {
                    //         return;
                    //     };
                    //     entry.get_or_insert_default().when = dt.and_utc();
                    // }
                    // CanMessage::BatteryChargeHistoryCharge(BatteryChargeHistoryCharge {
                    //     idx,
                    //     charge,
                    //     ..
                    // }) => {
                    //     let Some(entry) = self.charges.get_mut(*idx as usize) else {
                    //         return;
                    //     };
                    //     entry.get_or_insert_default().charge = *charge;
                    // }
        }
    }
}

#[cfg(feature = "app")]
impl PrivateState {
    fn update_from_can_message(&mut self, can_msg: &CanMessage) {
        match can_msg {
            CanMessage::ControllerSpeed(controller_speed) => {
                let current_speed = controller_speed.motor_speed;
                self.average_speed.feed(current_speed as u64);
            }
            CanMessage::BatteryVoltageCurrent(BatteryVoltageCurrent { current_ma, .. }) => {
                // battery current is negative when draining
                self.average_current
                    .feed((-current_ma).saturating_cast_unsigned() as u64)
            }
            _ => {}
        }
    }

    fn periodic_update(&mut self) {
        self.periodic_update_speed();
        self.periodic_update_range();
    }

    fn periodic_update_range(&mut self) {
        let (period, avg_current_ma) = self.average_current.take();

        let period_seconds = period.as_secs();

        let integrated: u16 = avg_current_ma
            .saturating_mul(period_seconds)
            .saturating_div(60)
            .saturating_truncate();

        defmt::debug!("Current integrated: {}", integrated);

        if integrated < 400 {
            defmt::trace!("Not updating range, current under limit");
            return;
        }

        let recent_avg_drain_ma_minutes: u16 = self.current_history.average(integrated);

        let battery_remaining_ma = read_state(|s| s.battery_info.absolute_soc);

        if recent_avg_drain_ma_minutes < 400 {
            defmt::trace!("Not updating range, current under limit");
            return;
        }

        let minutes_remaining = (battery_remaining_ma as u32)
            .saturating_mul(60)
            .saturating_div(recent_avg_drain_ma_minutes as u32);

        // our average speed is in km/h
        // minutes_ramining is m
        //
        // speed * minutes_remaining / 60 = km

        self.predicted_range = (self.moving_average_speed as u32 * minutes_remaining)
            .saturating_div(60)
            .saturating_truncate();
    }

    fn periodic_update_speed(&mut self) {
        // if the odometer is not yet loaded, defer to the next cycle
        let Some(current) = Odometer::maybe_get_stored() else {
            return;
        };

        // avg_speed is in km/h * 100, we want to store m as our odometer
        let (period, avg_speed) = self.average_speed.take();
        self.moving_average_speed = self
            .speed_history
            .average(avg_speed.saturating_div(100) as u16);

        let period_seconds = period.as_secs();

        // 24km/h         -> 24 km/h
        // * 10s (period) -> 24 km/h * 10s
        // * 1000         -> 24_000 m/h * 10s
        // / (60 * 60)    -> 6.6666 m/s * 10s
        //                -> 66.666 m
        // / 100          -> 0.6 (m/100)
        //
        // therefore:
        // (avg_speed * period_seconds * 1000) / (60 * 60 * 100)
        //
        // factor to:
        // (avg_speed * period_seconds) / (60 * 6)

        let integrated = avg_speed
            .saturating_mul(period_seconds)
            .saturating_div(60 * 6);

        defmt::debug!("Odo integrated: {}", integrated);

        Odometer::update_stored(Odometer {
            total_distance: current.total_distance.saturating_add(integrated as u32),
        });
    }

    fn update_public(&self, s: &mut SystemState) {
        // this is m
        if let Some(current) = Odometer::maybe_get_stored() {
            s.odometer = current.km() as u16;
        }

        s.predicted_range = self.predicted_range;
    }
}
