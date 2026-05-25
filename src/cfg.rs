use minicbor::{CborLen, Decode, Encode};

use crate::operation::DEFAULT_SPEED_LIMIT;

pub(crate) trait Storable:
    Default
    + PartialEq
    + Clone
    + minicbor::Encode<()>
    + for<'a> minicbor::Decode<'a, ()>
    + minicbor::CborLen<()>
    + 'static
{
    const ID: u8;

    fn take_if_changed() -> Option<Self>;
    fn update_stored(val: Self);
    async fn get_stored() -> Self;
}

macro_rules! saved_item {
    ($id:expr, $name:ident, $ty:ty) => {
        static $name: embassy_sync::blocking_mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            Option<($ty, bool)>,
        > = embassy_sync::blocking_mutex::Mutex::new(None);

        paste::paste! {
            static [<WAKER_ $name>]: embassy_sync::waitqueue::AtomicWaker = embassy_sync::waitqueue::AtomicWaker::new();

            impl Storable for $ty {
                const ID: u8 = $id;

                fn take_if_changed() -> Option<Self> {
                    unsafe {
                        $name.lock_mut(|s| {
                            let Some((x, v)) = s.as_mut() else {
                                return None;
                            };

                            if *v {
                                *v = false;
                                return Some(x.clone());
                            }

                            None
                        })
                    }
                }

                fn update_stored(val: Self) {
                    unsafe {
                        $name.lock_mut(|s| {
                            if let Some((prev, prev_changed)) = s.as_mut() {
                                if &val != prev {
                                    *prev_changed = true;
                                    *prev = val;
                                }
                            } else {
                                *s = Some((val, true));
                            }
                        })
                    }
                    [<WAKER_ $name>].wake();
                }

                async fn get_stored() -> Self {
                    core::future::poll_fn(|cx| {
                        if let Some((v, _)) = $name.lock(|s| s.clone()) {
                            core::task::Poll::Ready(v)
                        } else {
                            [<WAKER_ $name>].register(cx.waker());
                            core::task::Poll::Pending
                        }
                    })
                    .await
                }
            }
        }
    };
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, CborLen, defmt::Format)]
pub struct SpeedLimit {
    #[n(0)]
    limit: u8,
}

impl SpeedLimit {
    pub fn get_validated(self) -> u8 {
        if self.limit == 0 {
            DEFAULT_SPEED_LIMIT
        } else if self.limit > 45 {
            DEFAULT_SPEED_LIMIT
        } else {
            self.limit
        }
    }

    pub fn new_validated(val: u8) -> Self {
        let limit = if val == 0 {
            DEFAULT_SPEED_LIMIT
        } else if val > 45 {
            DEFAULT_SPEED_LIMIT
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

saved_item!(1, SPEED_LIMIT, SpeedLimit);
// pub static SPEED_LIMIT: StorageListNode<SpeedLimit> = StorageListNode::new("cfg/speed_limit");

#[derive(Encode, Decode, CborLen, defmt::Format, PartialEq, Eq, Copy, Clone, derive_enum_rotate::EnumRotate, Default)]
#[rustfmt::skip]
pub enum HeadlightMode {
    #[default]
    #[n(0)] Auto,
    #[n(1)] On,
    #[n(2)] Off,
}

saved_item!(2, HEADLIGHT_MODE, HeadlightMode);
// pub static HEADLIGHT_MODE: StorageListNode<HeadlightMode> =
//     StorageListNode::new("cfg/headlight_mode");

#[derive(Encode, Decode, CborLen, defmt::Format, PartialEq, Eq, Copy, Clone, derive_enum_rotate::EnumRotate, Default)]
#[rustfmt::skip]
pub enum SpeedMode {
    #[default]
    #[n(0)] Walk,
    #[n(1)] Eco,
    #[n(2)] Trip,
    #[n(3)] Sport,
}

saved_item!(3, SPEED_MODE, SpeedMode);
// pub static SPEED_MODE: StorageListNode<SpeedMode> = StorageListNode::new("cfg/speed_mode");

#[derive(Encode, Decode, CborLen, defmt::Format, PartialEq, Eq, Copy, Clone, Default)]
pub struct UnlockCode {
    #[n(0)]
    #[cbor(default)]
    pub digits: [crate::pin_digit::PinDigit; 4],
}

saved_item!(4, UNLOCK_CODE, UnlockCode);
