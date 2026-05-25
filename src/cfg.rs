use cfg_noodle::{
    StorageListNode,
    minicbor::{CborLen, Decode, Encode},
};

use crate::operation::DEFAULT_SPEED_LIMIT;

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

pub static SPEED_LIMIT: StorageListNode<SpeedLimit> = StorageListNode::new("cfg/speed_limit");

#[derive(Encode, Decode, CborLen, defmt::Format, PartialEq, Eq, Copy, Clone, derive_enum_rotate::EnumRotate, Default)]
#[rustfmt::skip]
pub enum HeadlightMode {
    #[default]
    #[n(0)] Auto,
    #[n(1)] On,
    #[n(2)] Off,
}

pub static HEADLIGHT_MODE: StorageListNode<HeadlightMode> =
    StorageListNode::new("cfg/headlight_mode");

#[derive(Encode, Decode, CborLen, defmt::Format, PartialEq, Eq, Copy, Clone, derive_enum_rotate::EnumRotate, Default)]
#[rustfmt::skip]
pub enum SpeedMode {
    #[default]
    #[n(0)] Walk,
    #[n(1)] Eco,
    #[n(2)] Trip,
    #[n(3)] Sport,
}

pub static SPEED_MODE: StorageListNode<SpeedMode> = StorageListNode::new("cfg/speed_mode");

#[derive(Encode, Decode, CborLen, defmt::Format, PartialEq, Eq, Copy, Clone, Default)]
pub struct UnlockCode {
    #[n(0)]
    #[cbor(default)]
    pub digits: [crate::pin_digit::PinDigit; 4],
}

pub static UNLOCK_CODE: StorageListNode<UnlockCode> = StorageListNode::new("cfg/unlock_code");
