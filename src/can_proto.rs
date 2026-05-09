use core::str::FromStr as _;

use heapless::LenType;

#[derive(defmt::Format, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanId {
    // Controller -> display
    ControllerStatus = 512,
    ControllerSpeed = 513,
    ControllerTempMotor = 514,
    ControllerSpeedMode = 515,
    ControllerSpeedLimit = 528,

    // Display -> controller
    DisplaySpeedMode = 768,
    DisplayThrottle = 774,
    DisplayFirmwareUpdate = 478,
    DisplayTriggerDump = 494,
    DisplayFwProgress = 1792,
    DisplayUnknown2 = 1856,
    DisplayUnknown3 = 1858,
    DisplaySerialRequest = 1860,
    DisplayChargeHistoryRequest = 1862,
    DisplayDiagnostics = 1904,
    DisplayDiagnostics2 = 1905,

    // Battery -> display
    BatteryCommandState = 1024,
    BatteryVoltageCurrent = 1025,
    BatteryChargeLevel = 1026,
    BatteryStateOfHealth = 1027,
    BatteryCapacityTemp = 1028,
    BatteryChargeHistory = 1863,

    // Other
    UpdateDevice = 900,

    // Not found in any firmware
    Unknown0x600 = 1536,
    Unknown0x601 = 1537,
    Unknown0x603 = 1539,
    Unknown0x606 = 1542,
    Unknown0x607 = 1543,
    Unknown0x7E0 = 1984,
    Unknown0x7F8 = 2040,
    Unknown0x238 = 568,
}

pub trait CanValue {
    fn can_id() -> CanId;
}

/// 512
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct ControllerStatus {
    #[deku(pad_bytes_after = "7")]
    pub battery_level: u8,
}

impl CanValue for ControllerStatus {
    fn can_id() -> CanId {
        CanId::ControllerStatus
    }
}

/// 513
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct ControllerSpeed {
    #[deku(endian = "little", pad_bytes_after = "2")]
    pub motor_speed: u16,
    pub status: u8,
}

impl ControllerSpeed {
    pub fn brake_light_on(&self) -> bool {
        self.status & 0x01 != 0
    }

    pub fn headlight_on(&self) -> bool {
        self.status & 0x02 != 0
    }

    pub fn walk_mode(&self) -> bool {
        self.status & 0x04 != 0
    }
}

impl CanValue for ControllerSpeed {
    fn can_id() -> CanId {
        CanId::ControllerSpeed
    }
}

// 514
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct ControllerTempMotor {
    #[deku(pad_bytes_before = "1")]
    pub temp: u8,

    #[deku(bytes = "3", endian = "little", pad_bytes_after = "4")]
    pub voltage: u32,
}

impl CanValue for ControllerTempMotor {
    fn can_id() -> CanId {
        CanId::ControllerTempMotor
    }
}

#[derive(defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpeedMode {
    Walk,
    Eco,
    Trip,
    Sport,
}

// 515
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct ControllerSpeedMode {
    #[deku(pad_bytes_before = "6")]
    pub idk: u16,
}

impl CanValue for ControllerSpeedMode {
    fn can_id() -> CanId {
        CanId::ControllerSpeedMode
    }
}

// 528
#[derive(deku::DekuRead, defmt::Format, Clone, Copy, PartialEq, Eq)]
pub struct ControllerSpeedLimit {
    #[deku(pad_bytes_after = "7")]
    pub speed_limit: bool,
}

impl CanValue for ControllerSpeedLimit {
    fn can_id() -> CanId {
        CanId::ControllerSpeedLimit
    }
}

// 768
#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
pub struct DisplaySpeedMode {
    /// Speed mode: 0x00=locked, 0x01=eco, 0x02=trip, 0x03=sport, 0x00+0xa5=walk
    pub mode: u8,
    /// Mode high byte
    pub mode_high: u8,
    /// Headlight: 0x64 = on, 0 = off
    pub headlight: u8,
    #[deku(magic = b"\x5a\x64\x00")]
    pub speed_mode_byte: u8,
    /// Walk mode counter (counts to 0xf)
    pub walk_counter: u8,
}

impl DisplaySpeedMode {
    pub fn new(mode: SpeedMode, headlight: bool) -> Self {
        let (mode_byte, mode_high, speed_mode_byte) = match mode {
            SpeedMode::Walk => (0x00, 0xa5, 0x00),
            SpeedMode::Eco => (0x01, 0x5a, 0x1e),
            SpeedMode::Trip => (0x02, 0x5a, 0x32),
            SpeedMode::Sport => (0x03, 0x5a, 0x46),
        };
        DisplaySpeedMode {
            mode: mode_byte,
            mode_high,
            headlight: if headlight { 0x64 } else { 0x00 },
            speed_mode_byte,
            walk_counter: 0,
        }
    }

    pub fn with_walk_counter(mut self, counter: u8) -> Self {
        self.walk_counter = counter;
        self
    }
}

impl CanValue for DisplaySpeedMode {
    fn can_id() -> CanId {
        CanId::DisplaySpeedMode
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, defmt::Format)]
struct DekuConst<const DATA: &'static [u8]>;

impl<Ctx, const DATA: &'static [u8]> deku::DekuWriter<Ctx> for DekuConst<DATA> {
    fn to_writer<W: deku::no_std_io::Write + deku::no_std_io::Seek>(
        &self,
        writer: &mut deku::writer::Writer<W>,
        ctx: Ctx,
    ) -> Result<(), deku::DekuError> {
        DATA.to_writer(writer, deku::ctx::ByteSize(1))
    }
}

impl<const DATA: &'static [u8]> deku::DekuSize for DekuConst<DATA> {
    const SIZE_BITS: usize = DATA.len() * 8;
}

// 774
#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
#[deku(bit_order = "lsb")]
pub struct DisplayThrottle {
    #[deku(bits = 9)]
    pub throttle: u16,

    #[deku(bits = 1)]
    pub left_blinker: bool,
    #[deku(bits = 1)]
    pub right_blinker: bool,

    #[deku(pad_bits_before = "5", pad_bytes_before = "1")]
    pub speed_limit: u8,

    pub magic: DekuConst<{ &[2, 0, 0, 0] }>,
}

impl DisplayThrottle {
    pub fn new(throttle: u16, left_blinker: bool, right_blinker: bool, speed_limit: u8) -> Self {
        DisplayThrottle {
            throttle,
            left_blinker,
            right_blinker,
            speed_limit,
            magic: DekuConst,
        }
    }
}

impl CanValue for DisplayThrottle {
    fn can_id() -> CanId {
        CanId::DisplayThrottle
    }
}

// 1862
#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
#[deku(magic = b"\x01\xfe")]
pub struct DisplayChargeHistoryRequest;

impl DisplayChargeHistoryRequest {
    pub fn new() -> Self {
        DisplayChargeHistoryRequest
    }
}

impl CanValue for DisplayChargeHistoryRequest {
    fn can_id() -> CanId {
        CanId::DisplayChargeHistoryRequest
    }
}

// 1024
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct BatteryCommandState {
    pub command: u16,
    pub state: u16,

    /// seems to always be zero
    pub estimated_range: u32,
}

impl CanValue for BatteryCommandState {
    fn can_id() -> CanId {
        CanId::BatteryCommandState
    }
}

// 1025
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
#[deku(endian = "little")]
pub struct BatteryVoltageCurrent {
    /// Voltage in mV (4 bytes)
    pub voltage_mv: u32,
    /// Current in mA (4 bytes, may be negative for discharge)
    pub current_ma: i32,
}

impl CanValue for BatteryVoltageCurrent {
    fn can_id() -> CanId {
        CanId::BatteryVoltageCurrent
    }
}

// 1026
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
#[deku(endian = "little")]
pub struct BatteryChargeLevel {
    /// Relative state of charge (%)
    pub relative_soc: u32,
    /// Absolute state of charge (mAh)
    pub absolute_soc_mah: u32,
}

impl CanValue for BatteryChargeLevel {
    fn can_id() -> CanId {
        CanId::BatteryChargeLevel
    }
}

// 1027
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
#[deku(endian = "little")]
pub struct BatteryStateOfHealth {
    /// Relative SOH (%)
    pub relative_soh: u8,
    /// Absolute SOH (mAh)
    #[deku(pad_bytes_before = "3")]
    pub absolute_soh_mah: u32,
}

impl CanValue for BatteryStateOfHealth {
    fn can_id() -> CanId {
        CanId::BatteryStateOfHealth
    }
}

// 1028
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
#[deku(endian = "little")]
pub struct BatteryCapacityTemp {
    /// Capacity: 0x4e20 = 20,000 mAh
    pub capacity_mah: u16,

    /// Battery charged flag (byte 4)
    #[deku(pad_bytes_before = "2")]
    pub battery_charged: bool,

    #[deku(pad_bytes_before = "2")]
    pub battery_temp_raw: u16,
}

impl BatteryCapacityTemp {
    pub fn battery_temp(&self) -> f32 {
        // Firmware stores (bytes 7&6 - 2731) / 10
        (self.battery_temp_raw as i16 as i32 - 2731) as f32 / 10.0
    }
}

impl CanValue for BatteryCapacityTemp {
    fn can_id() -> CanId {
        CanId::BatteryCapacityTemp
    }
}

// 1863
/// Single charge history entry pair (two messages come in pairs).
#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct BatteryChargeHistoryEntry {
    /// Index
    pub idx: u8,

    /// Year (2000+)
    #[deku(pad_bytes_before = "1")]
    pub year: u8,
    /// Month
    pub month: u8,
    /// Day
    pub day: u8,
    /// Hour
    pub hour: u8,
    /// Minute
    pub minute: u8,
    /// Second
    pub second: u8,
}

#[derive(deku::DekuRead, defmt::Format, Clone, PartialEq, Eq)]
pub struct BatteryChargeHistoryCharge {
    /// Index (matches entry)
    pub idx: u8,

    #[deku(pad_bytes_before = "1")]
    pub charge: u16,
}

impl CanValue for BatteryChargeHistoryEntry {
    fn can_id() -> CanId {
        CanId::BatteryChargeHistory
    }
}

impl CanValue for BatteryChargeHistoryCharge {
    fn can_id() -> CanId {
        CanId::BatteryChargeHistory
    }
}

#[derive(defmt::Format)]
pub enum CanMessage {
    // Controller -> display (decoders)
    ControllerStatus(ControllerStatus),
    ControllerSpeed(ControllerSpeed),
    ControllerTempMotor(ControllerTempMotor),
    ControllerSpeedMode(ControllerSpeedMode),
    ControllerSpeedLimit(ControllerSpeedLimit),

    // Battery -> display (decoders)
    BatteryCommandState(BatteryCommandState),
    BatteryVoltageCurrent(BatteryVoltageCurrent),
    BatteryChargeLevel(BatteryChargeLevel),
    BatteryStateOfHealth(BatteryStateOfHealth),
    BatteryCapacityTemp(BatteryCapacityTemp),
    BatteryChargeHistoryEntry(BatteryChargeHistoryEntry),
    BatteryChargeHistoryCharge(BatteryChargeHistoryCharge),
}

impl CanMessage {
    /// Try to decode a raw CAN frame into a typed message.
    pub fn from_can_frame(can_id: u16, data: &[u8]) -> Option<Self> {
        use deku::DekuContainerRead as _;
        match can_id {
            512 => ControllerStatus::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerStatus),
            513 => ControllerSpeed::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerSpeed),
            514 => ControllerTempMotor::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerTempMotor),
            515 => ControllerSpeedMode::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerSpeedMode),
            528 => ControllerSpeedLimit::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerSpeedLimit),
            1024 => BatteryCommandState::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryCommandState),
            1025 => BatteryVoltageCurrent::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryVoltageCurrent),
            1026 => BatteryChargeLevel::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryChargeLevel),
            1027 => BatteryStateOfHealth::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryStateOfHealth),
            1028 => BatteryCapacityTemp::from_bytes((&data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryCapacityTemp),
            _ => None,
        }
    }

    pub fn can_id(&self) -> u16 {
        match self {
            CanMessage::ControllerStatus(_) => 512,
            CanMessage::ControllerSpeed(_) => 513,
            CanMessage::ControllerTempMotor(_) => 514,
            CanMessage::ControllerSpeedMode(_) => 515,
            CanMessage::ControllerSpeedLimit(_) => 528,
            CanMessage::BatteryCommandState(_) => 1024,
            CanMessage::BatteryVoltageCurrent(_) => 1025,
            CanMessage::BatteryChargeLevel(_) => 1026,
            CanMessage::BatteryStateOfHealth(_) => 1027,
            CanMessage::BatteryCapacityTemp(_) => 1028,
            CanMessage::BatteryChargeHistoryEntry(_) => 1863,
            CanMessage::BatteryChargeHistoryCharge(_) => 1863,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
