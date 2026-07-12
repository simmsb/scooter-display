use embassy_time::{Duration, Instant};

use crate::{bluetooth_proto::BluetoothString, pin_digit::PinDigit};

pub(crate) trait Storable:
    Default + PartialEq + Clone + for<'a> sequential_storage::map::Value<'a> + 'static
{
    const ID: u8;

    fn take_if_changed_and_timedout() -> Option<Self>;
    fn mark_unchanged();
    fn update_stored(val: Self);
    async fn get_stored() -> Self;
    fn maybe_get_stored() -> Option<Self>;
}

macro_rules! saved_item {
    ($id:expr, $name:ident, $ty:ty, $timeout:literal) => {
        static $name: embassy_sync::blocking_mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            Option<($ty, bool, Instant)>,
        > = embassy_sync::blocking_mutex::Mutex::new(None);

        impl<'a> ::sequential_storage::map::PostcardValue<'a> for $ty {}

        paste::paste! {
            static [<WAKER_ $name>]: embassy_sync::waitqueue::AtomicWaker = embassy_sync::waitqueue::AtomicWaker::new();

            impl Storable for $ty {
                const ID: u8 = $id;

                fn take_if_changed_and_timedout() -> Option<Self> {
                    let now = Instant::now();
                    unsafe {
                        $name.lock_mut(|s| {
                            let Some((x, v, t)) = s.as_mut() else {
                                return None;
                            };

                            if *v && (now > *t) {
                                *v = false;
                                return Some(x.clone());
                            }

                            None
                        })
                    }
                }

                fn mark_unchanged() {
                    unsafe {
                        $name.lock_mut(|s| {
                            if let Some((_, v, _)) = s.as_mut() {
                                *v = false;
                            };
                        })
                    }
                }

                fn update_stored(val: Self) {
                    let now = Instant::now();
                    let t = now.saturating_add(Duration::from_secs($timeout));
                    unsafe {
                        $name.lock_mut(|s| {
                            if let Some((prev, prev_changed, prev_t)) = s.as_mut() {
                                if &val != prev {
                                    *prev_changed = true;
                                    *prev = val;
                                    *prev_t = if *prev_t < now { t } else { *prev_t };
                                }
                            } else {
                                *s = Some((val, true, t));
                            }
                        })
                    }
                    [<WAKER_ $name>].wake();
                }

                async fn get_stored() -> Self {
                    core::future::poll_fn(|cx| {
                        if let Some((v, _, _)) = $name.lock(|s| s.clone()) {
                            core::task::Poll::Ready(v)
                        } else {
                            [<WAKER_ $name>].register(cx.waker());
                            core::task::Poll::Pending
                        }
                    })
                    .await
                }

                fn maybe_get_stored() -> Option<Self> {
                    if let Some((v, _, _)) = $name.lock(|s| s.clone()) {
                        Some(v)
                    } else {
                        None
                    }
                }
            }
        }
    };
}

pub const DEFAULT_SPEED_LIMIT: u16 = 220;

#[derive(Copy, Clone, PartialEq, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct SpeedLimit {
    /// Speed limit, in km/h * 10 (220 = 22km/h)
    limit: u16,
}

impl SpeedLimit {
    /// Retrieve value, we can't trust the stored value to not have been
    /// corrupted, so validate it on load.
    pub fn get_validated(self) -> u16 {
        if self.limit < 30 {
            30
        } else if self.limit > 450 {
            450
        } else {
            self.limit
        }
    }

    pub fn new_validated(val: u16) -> Self {
        let limit = if val < 30 {
            30
        } else if val > 450 {
            450
        } else {
            val
        };

        Self { limit }
    }
}

impl Default for SpeedLimit {
    fn default() -> Self {
        Self {
            limit: DEFAULT_SPEED_LIMIT,
        }
    }
}

saved_item!(1, SPEED_LIMIT, SpeedLimit, 10);

#[derive(defmt::Format, PartialEq, Eq, Copy, Clone, derive_enum_rotate::EnumRotate, Default, serde::Serialize, serde::Deserialize)]
#[rustfmt::skip]
pub enum HeadlightMode {
    #[default]
    Auto,
    On,
    Off,
}

saved_item!(2, HEADLIGHT_MODE, HeadlightMode, 10);

#[derive(defmt::Format, PartialEq, Eq, Copy, Clone, derive_enum_rotate::EnumRotate, Default, serde::Serialize, serde::Deserialize)]
#[rustfmt::skip]
pub enum SpeedMode {
    #[default]
    Walk,
    Eco,
    Trip,
    Sport,
}

saved_item!(3, SPEED_MODE, SpeedMode, 10);

#[derive(defmt::Format, PartialEq, Eq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnlockCode {
    pub digits: [PinDigit; 4],
}

impl Default for UnlockCode {
    fn default() -> Self {
        Self {
            digits: [PinDigit::D2, PinDigit::D7, PinDigit::D0, PinDigit::D8],
        }
    }
}

impl UnlockCode {
    pub fn as_bt_string(&self) -> BluetoothString<4, u8> {
        let mut s = heapless::String::new();

        let _ = s.push(self.digits[0].as_char());
        let _ = s.push(self.digits[1].as_char());
        let _ = s.push(self.digits[2].as_char());
        let _ = s.push(self.digits[3].as_char());

        BluetoothString(s)
    }
}

saved_item!(4, UNLOCK_CODE, UnlockCode, 10);

#[derive(
    defmt::Format, PartialEq, Eq, Copy, Clone, Default, serde::Serialize, serde::Deserialize,
)]
pub struct Odometer {
    /// Total travelled distance, in meters
    pub total_distance: u32,
}

impl Odometer {
    pub fn km(&self) -> u32 {
        self.total_distance.saturating_div(1000)
    }
}

saved_item!(5, ODOMETER, Odometer, 300);

// NOTE: make sure an entry is added in noodle.rs
