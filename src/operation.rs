use core::sync::atomic::AtomicU8;

use embassy_futures::select;
use embassy_time::{Duration, WithTimeout};

use crate::{
    adc::{AmbientLight, Throttle},
    buttons_proto::Buttons,
    can::CAN_TX_BUS,
    can_proto::{DisplaySpeedMode, DisplayThrottle, SpeedMode},
};

pub static STATE_UPDATES: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    (),
    4,
> = embassy_sync::watch::Watch::new_with(());

static STATE: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    OperationState,
> = embassy_sync::blocking_mutex::Mutex::new(OperationState::DEFAULT);

pub fn read_state<T>(f: impl for<'a> FnOnce(&'a OperationState) -> T) -> T {
    STATE.lock(f)
}

fn update_state<T>(f: impl for<'a> FnOnce(&'a mut OperationState) -> T) -> T {
    unsafe { STATE.lock_mut(f) }
}

static SPEED_MODE_COUNTER: AtomicU8 = AtomicU8::new(0);

fn next_speed_mode_counter() -> u8 {
    let next = SPEED_MODE_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);

    next & 0xf
}

#[derive(PartialEq, Eq, defmt::Format)]
pub enum OperationCommand {
    Unlock,
    Lock,
    SetSpeedLimit(u8),
    SetSpeedMode(SpeedMode),
    SetHeadlightMode(HeadlightMode),
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub enum OperationState {
    Locked,
    Active(ActiveState),
}

impl OperationState {
    const DEFAULT: Self = Self::Locked;

    fn read_if_active<T>(&self, f: impl FnOnce(&ActiveState) -> T) -> Option<T> {
        if let Self::Active(active) = self {
            Some(f(active))
        } else {
            None
        }
    }

    fn update_if_active(&mut self, f: impl FnOnce(&mut ActiveState)) {
        if let Self::Active(active) = self {
            f(active);
        }
    }
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub enum HeadlightMode {
    Auto {
        /// Headlight will switch on when ambient light reads under this
        low: AmbientLight,

        /// Headlight will switch off when ambient light reads over this
        high: AmbientLight,

        currently_on: bool,
    },
    On,
    Off,
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct ActiveState {
    throttle: Throttle,

    /// Speed limit in km/h, we'll later use this to select the 25/35/45 limit
    /// sent to the controller
    speed_limit: u8,

    speed_mode: SpeedMode,

    headlight_mode: HeadlightMode,
}

impl ActiveState {
    fn headlight_on(&self) -> bool {
        match self.headlight_mode {
            HeadlightMode::Auto { currently_on, .. } => currently_on,
            HeadlightMode::On => true,
            HeadlightMode::Off => false,
        }
    }
}

#[embassy_executor::task]
pub async fn operation_task() {
    operation_task_().await
}

async fn operation_task_() {
    defmt::info!("Operation task startup");

    let mut send_can_messages_ticker = embassy_time::Ticker::every(Duration::from_millis(100));

    let mut throttle_readings = crate::adc::THROTTLE_READINGS.receiver().unwrap();

    let state_updates = STATE_UPDATES.sender();

    loop {
        match select::select(
            send_can_messages_ticker.next(),
            throttle_readings
                .changed()
                .with_timeout(Duration::from_secs(1)),
        )
        .await
        {
            select::Either::First(_) => {
                send_speed_and_throttle_can_messages().await;
            }
            select::Either::Second(Ok(throttle)) => {
                update_state(|s| s.update_if_active(|a| a.throttle = throttle));

                state_updates.send(());
            }
            select::Either::Second(Err(_)) => {
                panic!("Operation task did not receive throttle update in time");
            }
        }
    }
}

async fn send_speed_and_throttle_can_messages() {
    let buttons = crate::state::read_state(|s| s.buttons);

    if let Some((throttle, speed_limit, speed_mode, headlight)) = read_state(|s| {
        s.read_if_active(|a| (a.throttle, a.speed_limit, a.speed_mode, a.headlight_on()))
    }) {
        let mut speed_mode_msg = DisplaySpeedMode::new(speed_mode, headlight);

        if speed_mode == SpeedMode::Walk {
            speed_mode_msg = speed_mode_msg.with_walk_counter(next_speed_mode_counter());
        }

        CAN_TX_BUS
            .send(crate::can_proto::Sent::DisplaySpeedMode(speed_mode_msg))
            .await;

        let speed_limit = match speed_limit {
            0..=25 => 0,
            26..=35 => 1,
            _ => 2,
        };

        let throttle_msg = DisplayThrottle::new(
            throttle.0,
            buttons.contains(Buttons::L_BLINK),
            buttons.contains(Buttons::R_BLINK),
            speed_limit,
        );

        CAN_TX_BUS
            .send(crate::can_proto::Sent::DisplayThrottle(throttle_msg))
            .await;
    } else {
        CAN_TX_BUS
            .send(crate::can_proto::Sent::DisplaySpeedMode(
                DisplaySpeedMode::immobile(next_speed_mode_counter()),
            ))
            .await;
    }
}
