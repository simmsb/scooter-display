use core::sync::atomic::AtomicU8;

use embassy_futures::select;
use embassy_time::{Duration, WithTimeout};

use crate::{
    adc::{AmbientLight, Throttle},
    buttons_proto::Buttons,
    can::CAN_TX_BUS,
    can_proto::{DisplaySpeedMode, DisplayThrottle},
    cfg::{HeadlightMode, SpeedLimit, SpeedMode, Storable, UnlockCode},
};

pub const DEFAULT_SPEED_LIMIT: u8 = 22;

pub static STATE_UPDATES: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    (),
    4,
> = embassy_sync::watch::Watch::new_with(());

static STATE: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    OperationState,
> = embassy_sync::blocking_mutex::Mutex::new(OperationState::DEFAULT);

pub fn read_state<T>(f: impl for<'a> FnOnce(&'a OperationState) -> T) -> T {
    STATE.lock(f)
}

fn update_state<T>(f: impl for<'a> FnOnce(&'a mut OperationState) -> T) -> T {
    unsafe { STATE.lock_mut(f) }
}

pub static OPERATION_COMMANDS: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    OperationCommand,
    1,
> = embassy_sync::channel::Channel::new();

static SPEED_MODE_COUNTER: AtomicU8 = AtomicU8::new(0);

fn next_speed_mode_counter() -> u8 {
    let next = SPEED_MODE_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);

    next & 0xf
}

#[derive(PartialEq, Eq, defmt::Format, Clone, Copy)]
pub enum OperationCommand {
    Unlock,
    Lock,
    SetSpeedLimit(u8),
    SetSpeedMode(SpeedMode),
    SetHeadlightMode(HeadlightMode),
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub enum OperationState {
    Locked(Option<UnlockCode>),
    Active(ActiveState),
}

impl OperationState {
    const DEFAULT: Self = Self::Locked(None);

    fn read_if_active<T>(&self, f: impl FnOnce(&ActiveState) -> T) -> Option<T> {
        if let Self::Active(active) = self {
            Some(f(active))
        } else {
            None
        }
    }

    pub fn as_locked(&self) -> Option<&Option<UnlockCode>> {
        if let OperationState::Locked(code) = self {
            Some(code)
        } else {
            None
        }
    }

    pub fn as_active(&self) -> Option<&ActiveState> {
        if let OperationState::Active(active) = self {
            Some(active)
        } else {
            None
        }
    }

    fn update_if_active(&mut self, f: impl FnOnce(&mut ActiveState)) {
        if let Self::Active(active) = self {
            f(active);
        }
    }

    pub fn is_locked(&self) -> bool {
        match self {
            Self::Locked(_) => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, defmt::Format, Copy, Clone)]
pub struct HeadlightConfig {
    /// Headlight will switch on when ambient light reads under this
    pub low: AmbientLight,

    /// Headlight will switch off when ambient light reads over this
    pub high: AmbientLight,

    pub auto_on: bool,
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct ActiveState {
    pub throttle: Throttle,

    /// Speed limit in km/h, we'll later use this to select the 25/35/45 limit
    /// sent to the controller
    pub speed_limit: u8,

    pub speed_mode: SpeedMode,

    pub headlight_mode: HeadlightMode,
    pub headlight_config: HeadlightConfig,
}

impl ActiveState {
    pub fn headlight_on(&self) -> bool {
        self.headlight_mode == HeadlightMode::On || self.headlight_config.auto_on
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
    let mut ambient_readings = crate::adc::AMBIENT_READINGS.receiver().unwrap();

    let operation_commands = OPERATION_COMMANDS.receiver();

    let state_updates = STATE_UPDATES.sender();

    let unlock_code = UnlockCode::get_stored().await;
    defmt::info!("Loaded unlock code: {}", unlock_code);

    update_state(|s| if s.is_locked() {
        *s = OperationState::Locked(Some(unlock_code));
    });

    state_updates.send(());

    loop {
        match select::select4(
            send_can_messages_ticker.next(),
            throttle_readings
                .changed()
                .with_timeout(Duration::from_secs(1)),
            ambient_readings.changed(),
            operation_commands.receive(),
        )
        .await
        {
            select::Either4::First(_) => {
                send_speed_and_throttle_can_messages().await;
            }
            select::Either4::Second(Ok(throttle)) => {
                update_state(|s| s.update_if_active(|a| a.throttle = throttle));

                state_updates.send(());
            }
            select::Either4::Second(Err(_)) => {
                panic!("Operation task did not receive throttle update in time");
            }
            select::Either4::Third(ambient) => update_state(|s| {
                s.update_if_active(|a| {
                    if a.headlight_mode == HeadlightMode::Auto {
                        if a.headlight_config.auto_on && ambient < a.headlight_config.low {
                            a.headlight_config.auto_on = true;
                        } else if !a.headlight_config.auto_on && ambient > a.headlight_config.high {
                            a.headlight_config.auto_on = false;
                        }
                    }
                })
            }),
            select::Either4::Fourth(op_cmd) => {
                defmt::info!("Handling op command: {}", op_cmd);
                match op_cmd {
                    OperationCommand::Unlock => {
                        let speed_limit = SpeedLimit::get_stored().await.get_validated();
                        let speed_mode = SpeedMode::get_stored().await;
                        let headlight_mode = HeadlightMode::get_stored().await;

                        update_state(|s: &mut OperationState| {
                            *s = OperationState::Active(ActiveState {
                                throttle: Throttle(0),
                                speed_limit,
                                speed_mode,
                                headlight_mode,
                                headlight_config: HeadlightConfig {
                                    low: AmbientLight(10),
                                    high: AmbientLight(30),
                                    auto_on: false,
                                },
                            })
                        })
                    },
                    OperationCommand::Lock => {
                        let unlock_code = UnlockCode::get_stored().await;
                        update_state(|s| *s = OperationState::Locked(Some(unlock_code)))
                    }
                    OperationCommand::SetSpeedLimit(new_limit) => {
                        SpeedLimit::update_stored(SpeedLimit::new_validated(new_limit));
                        update_state(|s| s.update_if_active(|a| a.speed_limit = new_limit))
                    }
                    OperationCommand::SetSpeedMode(speed_mode) => {
                        SpeedMode::update_stored(speed_mode);
                        update_state(|s| s.update_if_active(|a| a.speed_mode = speed_mode))
                    }
                    OperationCommand::SetHeadlightMode(headlight_mode) => {
                        HeadlightMode::update_stored(headlight_mode);
                        update_state(|s| {
                            s.update_if_active(|a| {
                                a.headlight_mode = headlight_mode;
                            })
                        })
                    },
                }

                defmt::info!("Handled op command");
                state_updates.send(());
            }
        }
    }
}

async fn send_speed_and_throttle_can_messages() {
    let buttons = crate::system_state::read_state(|s| s.buttons);

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

        defmt::trace!("Sending thottle: {}", throttle_msg);

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
