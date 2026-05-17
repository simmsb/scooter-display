use deku::{DekuContainerWrite as _, DekuSize};

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
}

// this is a bit wonky, but CAN IDs don't alias, just some are extended...
impl CanId {
    pub fn to_standard_raw(self) -> u16 {
        self as u16
    }

    pub fn to_extended_raw(self) -> u32 {
        self as u32
    }

    pub fn is_extended(self) -> bool {
        match self {
            Self::DisplayChargeHistoryRequest => true,
            _ => false,
        }
    }
}

pub trait CanValue {
    fn can_id() -> CanId;
}

/// 512
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
pub struct ControllerStatus {
    #[deku(pad_bytes_after = "5")]
    pub battery_level: u8,
    #[deku(pad_bytes_after = "1")]
    pub status: u8,
}

impl CanValue for ControllerStatus {
    fn can_id() -> CanId {
        CanId::ControllerStatus
    }
}

/// 513
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
#[deku(bit_order = "lsb", endian = "little")]
pub struct ControllerSpeed {
    #[deku(pad_bytes_after = "2")]
    pub motor_speed: u16,

    #[deku(bits = 1)]
    pub walk_mode: bool,

    #[deku(bits = 1)]
    pub headlight_on: bool,

    #[deku(bits = 1, pad_bits_after = "5")]
    pub brake_light_on: bool,
}

impl CanValue for ControllerSpeed {
    fn can_id() -> CanId {
        CanId::ControllerSpeed
    }
}

struct OffsetU8;

impl OffsetU8 {
    fn read<R: deku::no_std_io::Read + deku::no_std_io::Seek>(
        reader: &mut deku::reader::Reader<R>,
        offset: i8,
    ) -> Result<u8, deku::DekuError> {
        let value = <u8 as deku::DekuReader>::from_reader_with_ctx(reader, ())?;
        Ok(value.saturating_add_signed(offset))
    }

    #[allow(unused)]
    fn write<W: deku::no_std_io::Write + deku::no_std_io::Seek>(
        writer: &mut deku::writer::Writer<W>,
        value: u8,
        offset: i8,
    ) -> Result<(), deku::DekuError> {
        use deku::DekuWriter;

        value.saturating_add_signed(-offset).to_writer(writer, ())
    }
}

// 514
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
#[deku(endian = "little")]
pub struct ControllerTempMotor {
    #[deku(
        pad_bytes_before = "1",
        reader = "OffsetU8::read(deku::reader, 40i8)",
        writer = "OffsetU8::write(deku::writer, self.temp, 40i8)"
    )]
    pub temp: u8,

    #[deku(bytes = "2", pad_bytes_after = "4")]
    pub voltage: u16,
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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
#[deku(endian = "little")]
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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
pub struct ControllerSpeedLimit {
    #[deku(pad_bytes_after = "7")]
    pub speed_limit: bool,
}

impl CanValue for ControllerSpeedLimit {
    fn can_id() -> CanId {
        CanId::ControllerSpeedLimit
    }
}

pub enum Sent {
    DisplaySpeedMode(DisplaySpeedMode),
    DisplayThrottle(DisplayThrottle),
    DisplayChargeHistoryRequest(DisplayChargeHistoryRequest),
}

impl Sent {
    pub fn can_id(&self) -> CanId {
        match self {
            Sent::DisplaySpeedMode(_) => DisplaySpeedMode::can_id(),
            Sent::DisplayThrottle(_) => DisplayThrottle::can_id(),
            Sent::DisplayChargeHistoryRequest(_) => DisplayChargeHistoryRequest::can_id(),
        }
    }

    pub fn serialise<'buf>(&self, buf: &'buf mut [u8; 8]) -> &'buf [u8] {
        match self {
            Sent::DisplaySpeedMode(display_speed_mode) => {
                let buf = &mut buf[0..DisplaySpeedMode::SIZE_BYTES.unwrap()];
                display_speed_mode.to_slice(buf).unwrap();
                buf
            }
            Sent::DisplayThrottle(display_throttle) => {
                let buf = &mut buf[0..DisplayThrottle::SIZE_BYTES.unwrap()];
                display_throttle.to_slice(buf).unwrap();
                buf
            }
            Sent::DisplayChargeHistoryRequest(display_charge_history_request) => {
                let buf = &mut buf[0..DisplayChargeHistoryRequest::SIZE_BYTES.unwrap()];
                display_charge_history_request.to_slice(buf).unwrap();
                buf
            }
        }
    }
}

#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(test, derive(deku::DekuRead))]
#[deku(id_type = "u32", bytes = 3, endian = "big")]
pub enum DisplaySpeedModeMagic {
    #[deku(id = 0x5a6400)]
    Normal,

    /// Sending this triggers a controller shutdown
    #[deku(id = 0xa56400)]
    Shutdown,
}

// 768
#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(test, derive(deku::DekuRead))]
pub struct DisplaySpeedMode {
    /// Speed mode: 0x00=locked, 0x01=eco, 0x02=trip, 0x03=sport, 0x00+0xa5=walk
    /// Controller supports up to 0x9 for some reason?
    pub mode: u8,
    /// Mode high byte
    pub mode_high: u8,
    /// Headlight: 0x64 = on, 0 = off
    pub headlight: u8,
    pub magic: DisplaySpeedModeMagic,
    pub speed_mode_byte: u8,
    /// Walk mode counter (counts to 0xf)
    pub walk_counter: u8,
}

impl DisplaySpeedMode {
    pub const fn immobile(counter: u8) -> Self {
        Self {
            mode: 0,
            mode_high: 0x5a,
            headlight: 0,
            magic: DisplaySpeedModeMagic::Normal,
            speed_mode_byte: 0x64,
            walk_counter: counter,
        }
    }

    pub const fn shutdown() -> Self {
        Self {
            mode: 0,
            mode_high: 0x5a,
            headlight: 0,
            magic: DisplaySpeedModeMagic::Shutdown,
            speed_mode_byte: 0x64,
            walk_counter: 0,
        }
    }

    pub const fn new(mode: SpeedMode, headlight: bool) -> Self {
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
            magic: DisplaySpeedModeMagic::Normal,
            speed_mode_byte,
            walk_counter: 0,
        }
    }

    pub const fn with_custom_speed_mode(mut self, speed_mode: u8) -> Self {
        self.mode = speed_mode;
        self
    }

    pub const fn with_walk_counter(mut self, counter: u8) -> Self {
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
pub struct DekuConst<const DATA: &'static [u8]>;

impl<Ctx, const DATA: &'static [u8]> deku::DekuWriter<Ctx> for DekuConst<DATA> {
    fn to_writer<W: deku::no_std_io::Write + deku::no_std_io::Seek>(
        &self,
        writer: &mut deku::writer::Writer<W>,
        _ctx: Ctx,
    ) -> Result<(), deku::DekuError> {
        DATA.to_writer(writer, deku::ctx::ByteSize(1))
    }
}

impl<'a, Ctx, const DATA: &'static [u8]> deku::DekuReader<'a, Ctx> for DekuConst<DATA> {
    fn from_reader_with_ctx<R: deku::no_std_io::Read + deku::no_std_io::Seek>(
        reader: &mut deku::reader::Reader<R>,
        _ctx: Ctx,
    ) -> Result<Self, deku::DekuError>
    where
        Self: Sized,
    {
        for expected in DATA {
            let got = u8::from_reader_with_ctx(reader, deku::ctx::ByteSize(1))?;

            if *expected != got {
                return Err(deku::deku_error!(
                    deku::DekuError::Parse,
                    "Constant value mismatch",
                    "expected {} got {}",
                    expected,
                    got
                ));
            }
        }

        Ok(Self)
    }
}

impl<const DATA: &'static [u8]> deku::DekuSize for DekuConst<DATA> {
    const SIZE_BITS: usize = DATA.len() * 8;
}

// 774
#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(test, derive(deku::DekuRead))]
#[deku(bit_order = "lsb", endian = "little")]
pub struct DisplayThrottle {
    #[deku(bits = 9)]
    pub throttle: u16,

    #[deku(bits = 1)]
    pub left_blinker: bool,
    #[deku(bits = 1, pad_bits_after = "5")]
    pub right_blinker: bool,

    #[deku(pad_bytes_before = "1")]
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
#[cfg_attr(test, derive(deku::DekuRead))]
#[deku(magic = b"\x01\xfe")]
pub struct DisplayChargeHistoryRequest;

impl Default for DisplayChargeHistoryRequest {
    fn default() -> Self {
        Self::new()
    }
}

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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
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
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
#[deku(endian = "little")]
pub struct BatteryCapacityTemp {
    /// Capacity: 0x4e20 = 20,000 mAh
    pub capacity_mah: u16,

    /// Battery charged flag
    #[deku(pad_bytes_before = "2")]
    pub battery_charged: bool,

    /// Stored as celcius * 10
    #[deku(
        pad_bytes_before = "1",
        map = "|x: u16| -> Result<_, deku::DekuError> { Ok(x as i16 - 2731) }",
        writer = "(self.battery_temp + 2731).to_writer(deku::writer, ())"
    )]
    pub battery_temp: i16,
}

impl CanValue for BatteryCapacityTemp {
    fn can_id() -> CanId {
        CanId::BatteryCapacityTemp
    }
}

// 1863
/// Single charge history entry pair (two messages come in pairs).
#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
pub struct BatteryChargeHistoryEntry {
    /// Index
    pub idx: u8,

    /// Unknown value, firmware stores it but never reads.
    pub unknown: u8,

    /// Year (2000+)
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

#[derive(deku::DekuRead, deku::DekuSize, defmt::Format, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(deku::DekuWrite, Debug))]
pub struct BatteryChargeHistoryCharge {
    /// Index (matches entry)
    pub idx: u8,

    /// Unknown value, firmware stores it but never reads.
    pub unknown: u8,

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
    pub fn from_can_frame(can_id: u32, data: &[u8]) -> Option<Self> {
        use deku::DekuContainerRead as _;
        match can_id {
            512 => ControllerStatus::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerStatus),
            513 => ControllerSpeed::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerSpeed),
            514 => ControllerTempMotor::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerTempMotor),
            515 => ControllerSpeedMode::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerSpeedMode),
            528 => ControllerSpeedLimit::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::ControllerSpeedLimit),
            1024 => BatteryCommandState::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryCommandState),
            1025 => BatteryVoltageCurrent::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryVoltageCurrent),
            1026 => BatteryChargeLevel::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryChargeLevel),
            1027 => BatteryStateOfHealth::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryStateOfHealth),
            1028 => BatteryCapacityTemp::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryCapacityTemp),
            1863 if data.len() == 8 => BatteryChargeHistoryEntry::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryChargeHistoryEntry),
            1863 if data.len() == 4 => BatteryChargeHistoryCharge::from_bytes((data, 0))
                .ok()
                .map(|(_, x)| x)
                .map(CanMessage::BatteryChargeHistoryCharge),
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

    pub fn deser_roundtrip<
        T: deku::DekuSize
            + for<'a> TryFrom<&'a [u8], Error = E>
            + deku::DekuContainerWrite
            + std::fmt::Debug
            + PartialEq,
        E: std::fmt::Debug,
    >(
        buf: &mut [u8],
        val: &T,
    ) {
        assert_eq!(
            buf.len(),
            T::SIZE_BYTES.unwrap(),
            "Buf should match expected size, given: {}, expected: {}",
            buf.len(),
            T::SIZE_BYTES.unwrap()
        );

        let n = val.to_slice(&mut buf[..T::SIZE_BYTES.unwrap()]).unwrap();
        assert_eq!(
            n,
            T::SIZE_BYTES.unwrap(),
            "Should serialize to its expected size"
        );

        let parsed = T::try_from(buf).unwrap();

        assert_eq!(val, &parsed);
    }

    pub fn serde_roundtrip<
        T: deku::DekuSize
            + for<'a> TryFrom<&'a [u8], Error = E>
            + deku::DekuContainerWrite
            + std::fmt::Debug
            + PartialEq,
        E: std::fmt::Debug,
    >(
        buf: &[u8],
    ) -> T {
        let parsed = T::try_from(&buf[..T::SIZE_BYTES.unwrap()]).unwrap();
        let d_buf = &mut [0u8; 8][..T::SIZE_BYTES.unwrap()];

        let n = parsed.to_slice(d_buf).unwrap();
        assert_eq!(
            n,
            T::SIZE_BYTES.unwrap(),
            "Should serialize to its expected size"
        );

        assert_eq!(
            buf, d_buf,
            "Bad serde roundtrip, in: {:?}, out: {:?}, parsed: {:?}",
            buf, d_buf, parsed
        );

        parsed
    }

    #[test]
    fn test_controller_status() {
        let msg = serde_roundtrip::<ControllerStatus, _>(&[0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            ControllerStatus {
                battery_level: 0,
                status: 0
            }
        );

        let msg = serde_roundtrip::<ControllerStatus, _>(&[100, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            ControllerStatus {
                battery_level: 100,
                status: 0
            }
        );

        let msg = serde_roundtrip::<ControllerStatus, _>(&[100, 0, 0, 0, 0, 0, 2, 0]);
        assert_eq!(
            msg,
            ControllerStatus {
                battery_level: 100,
                status: 2
            }
        );
    }

    #[test]
    fn test_display_speed_mode() {
        let mut buf = [0u8; 8];
        deser_roundtrip(&mut buf, &DisplaySpeedMode::new(SpeedMode::Trip, true));
        assert_eq!(buf, [0x02, 0x5a, 0x64, 0x5a, 0x64, 0, 0x32, 0]);

        deser_roundtrip(&mut buf, &DisplaySpeedMode::new(SpeedMode::Eco, true));
        assert_eq!(buf, [0x01, 0x5a, 0x64, 0x5a, 0x64, 0, 0x1e, 0]);

        deser_roundtrip(&mut buf, &DisplaySpeedMode::new(SpeedMode::Walk, false));
        assert_eq!(buf, [0x00, 0xa5, 0x00, 0x5a, 0x64, 0, 0, 0]);

        deser_roundtrip(
            &mut buf,
            &DisplaySpeedMode::new(SpeedMode::Walk, false).with_walk_counter(2),
        );
        assert_eq!(buf, [0x00, 0xa5, 0x00, 0x5a, 0x64, 0, 0, 2]);

        deser_roundtrip(&mut buf, &DisplaySpeedMode::immobile(4));
        assert_eq!(buf, [0x00, 0x5a, 0x00, 0x5a, 0x64, 0, 0x64, 4]);

        deser_roundtrip(&mut buf, &DisplaySpeedMode::shutdown());
        assert_eq!(buf, [0x00, 0x5a, 0x00, 0xa5, 0x64, 0, 0x64, 0]);
    }

    #[test]
    fn test_controller_speed() {
        let msg = serde_roundtrip::<ControllerSpeed, _>(&[0, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            ControllerSpeed {
                motor_speed: 0,
                walk_mode: false,
                headlight_on: false,
                brake_light_on: false
            }
        );

        let msg = serde_roundtrip::<ControllerSpeed, _>(&[7, 4, 0, 0, 0]);
        assert_eq!(
            msg,
            ControllerSpeed {
                motor_speed: 1031,
                walk_mode: false,
                headlight_on: false,
                brake_light_on: false
            }
        );

        let msg = serde_roundtrip::<ControllerSpeed, _>(&[7, 4, 0, 0, 0b1]);
        assert_eq!(
            msg,
            ControllerSpeed {
                motor_speed: 1031,
                walk_mode: true,
                headlight_on: false,
                brake_light_on: false
            }
        );

        let msg = serde_roundtrip::<ControllerSpeed, _>(&[7, 4, 0, 0, 0b100]);
        assert_eq!(
            msg,
            ControllerSpeed {
                motor_speed: 1031,
                walk_mode: false,
                headlight_on: false,
                brake_light_on: true
            }
        );

        let msg = serde_roundtrip::<ControllerSpeed, _>(&[7, 4, 0, 0, 0b111]);
        assert_eq!(
            msg,
            ControllerSpeed {
                motor_speed: 1031,
                walk_mode: true,
                headlight_on: true,
                brake_light_on: true
            }
        );
    }

    #[test]
    fn test_controller_temp_motor() {
        let msg = serde_roundtrip::<ControllerTempMotor, _>(&[0; 8]);
        assert_eq!(
            msg,
            ControllerTempMotor {
                temp: 40,
                voltage: 0
            }
        );

        let msg = serde_roundtrip::<ControllerTempMotor, _>(&[0, 0x11, 0x1c, 0xd4, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            ControllerTempMotor {
                temp: 57,
                voltage: 54300
            }
        );

        let msg = serde_roundtrip::<ControllerTempMotor, _>(&[0, 0x13, 0xe4, 0xd4, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            ControllerTempMotor {
                temp: 59,
                voltage: 54500
            }
        );
    }

    #[test]
    fn test_controller_speed_mode() {
        let msg = serde_roundtrip::<ControllerSpeedMode, _>(&[0; 8]);
        assert_eq!(msg, ControllerSpeedMode { idk: 0 });

        let msg = serde_roundtrip::<ControllerSpeedMode, _>(&[0, 0, 0, 0, 0, 0, 0x43, 0x12]);
        assert_eq!(msg, ControllerSpeedMode { idk: 0x1243 });
    }

    #[test]
    fn test_controller_speed_limit() {
        let msg = serde_roundtrip::<ControllerSpeedLimit, _>(&[0; 8]);
        assert_eq!(msg, ControllerSpeedLimit { speed_limit: false });

        let msg = serde_roundtrip::<ControllerSpeedLimit, _>(&[1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(msg, ControllerSpeedLimit { speed_limit: true });
    }

    #[test]
    fn test_battery_command_state() {
        let msg = serde_roundtrip::<BatteryCommandState, _>(&[1, 0, 0xc, 0, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            BatteryCommandState {
                command: 1,
                state: 12,
                estimated_range: 0
            }
        );

        let msg = serde_roundtrip::<BatteryCommandState, _>(&[1, 0x40, 0x2c, 0x40, 0, 0, 0, 0]);
        assert_eq!(
            msg,
            BatteryCommandState {
                command: 0b100000000000001,
                state: 0b100000000101100,
                estimated_range: 0
            }
        );
    }

    #[test]
    fn test_battery_voltage_current() {
        let msg = serde_roundtrip::<BatteryVoltageCurrent, _>(&[
            0x00, 0xc5, 0x00, 0x00, 0xc8, 0xfc, 0xff, 0xff,
        ]);
        assert_eq!(
            msg,
            BatteryVoltageCurrent {
                voltage_mv: 50432,
                current_ma: -824
            }
        );

        let msg = serde_roundtrip::<BatteryVoltageCurrent, _>(&[
            0x00, 0xd3, 0x00, 0x00, 0x9f, 0xff, 0xff, 0xff,
        ]);
        assert_eq!(
            msg,
            BatteryVoltageCurrent {
                voltage_mv: 54016,
                current_ma: -97
            }
        );

        let msg = serde_roundtrip::<BatteryVoltageCurrent, _>(&[
            0x00, 0xd4, 0x00, 0x00, 0x2c, 0x05, 0x00, 0x00,
        ]);
        assert_eq!(
            msg,
            BatteryVoltageCurrent {
                voltage_mv: 54272,
                current_ma: 1324
            }
        );
    }

    #[test]
    fn test_battery_charge_level() {
        let msg = serde_roundtrip::<BatteryChargeLevel, _>(&[0x3f, 0, 0, 0, 0xbf, 0x31, 0, 0]);
        assert_eq!(
            msg,
            BatteryChargeLevel {
                relative_soc: 63,
                absolute_soc_mah: 12735
            }
        );

        let msg = serde_roundtrip::<BatteryChargeLevel, _>(&[0x62, 0, 0, 0, 0x46, 0x4d, 0, 0]);
        assert_eq!(
            msg,
            BatteryChargeLevel {
                relative_soc: 98,
                absolute_soc_mah: 19782
            }
        );

        let msg = serde_roundtrip::<BatteryChargeLevel, _>(&[0x64, 0, 0, 0, 0x77, 0x4d, 0, 0]);
        assert_eq!(
            msg,
            BatteryChargeLevel {
                relative_soc: 100,
                absolute_soc_mah: 19831
            }
        );
    }

    #[test]
    fn test_battery_state_of_health() {
        let msg = serde_roundtrip::<BatteryStateOfHealth, _>(&[0x64, 0, 0, 0, 0x80, 0x4d, 0, 0]);
        assert_eq!(
            msg,
            BatteryStateOfHealth {
                relative_soh: 100,
                absolute_soh_mah: 19840
            }
        );

        let msg = serde_roundtrip::<BatteryStateOfHealth, _>(&[0x64, 0, 0, 0, 0x20, 0x4e, 0, 0]);
        assert_eq!(
            msg,
            BatteryStateOfHealth {
                relative_soh: 100,
                absolute_soh_mah: 20000
            }
        );
    }

    #[test]
    fn test_battery_capacity_temp() {
        let msg = serde_roundtrip::<BatteryCapacityTemp, _>(&[0x20, 0x4e, 0, 0, 1, 0, 0xbe, 0x0b]);
        assert_eq!(
            msg,
            BatteryCapacityTemp {
                capacity_mah: 20000,
                battery_charged: true,
                battery_temp: 275
            }
        );

        let msg = serde_roundtrip::<BatteryCapacityTemp, _>(&[0x20, 0x4e, 0, 0, 0, 0, 0x91, 0x0b]);
        assert_eq!(
            msg,
            BatteryCapacityTemp {
                capacity_mah: 20000,
                battery_charged: false,
                battery_temp: 230
            }
        );
    }

    #[test]
    fn test_battery_charge_history_entry() {
        let msg = serde_roundtrip::<BatteryChargeHistoryEntry, _>(&[
            0x0b, 0x01, 0x19, 0x08, 0x1a, 0x12, 0x20, 0x1c,
        ]);
        assert_eq!(
            msg,
            BatteryChargeHistoryEntry {
                idx: 11,
                unknown: 1,
                year: 25,
                month: 8,
                day: 26,
                hour: 18,
                minute: 32,
                second: 28
            }
        );

        let msg = serde_roundtrip::<BatteryChargeHistoryEntry, _>(&[
            0x04, 0x02, 0x19, 0x08, 0x14, 0x06, 0x0f, 0x24,
        ]);
        assert_eq!(
            msg,
            BatteryChargeHistoryEntry {
                idx: 4,
                unknown: 2,
                year: 25,
                month: 8,
                day: 20,
                hour: 6,
                minute: 15,
                second: 36
            }
        );
    }

    #[test]
    fn test_battery_charge_history_charge() {
        let msg = serde_roundtrip::<BatteryChargeHistoryCharge, _>(&[0xa, 0x02, 0x5d, 0x4e]);
        assert_eq!(
            msg,
            BatteryChargeHistoryCharge {
                idx: 10,
                unknown: 2,
                charge: 20061
            }
        );

        let msg = serde_roundtrip::<BatteryChargeHistoryCharge, _>(&[0x1, 0x01, 0x7a, 0x07]);
        assert_eq!(
            msg,
            BatteryChargeHistoryCharge {
                idx: 1,
                unknown: 1,
                charge: 1914
            }
        );
    }

    #[test]
    fn test_display_throttle() {
        let mut buf = [0u8; 8];
        deser_roundtrip(&mut buf, &DisplayThrottle::new(511, false, false, 0));
        assert_eq!(buf, [0xff, 0b1, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]);

        deser_roundtrip(&mut buf, &DisplayThrottle::new(511, true, false, 0));
        assert_eq!(buf, [0xff, 0b011, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]);

        deser_roundtrip(&mut buf, &DisplayThrottle::new(511, true, true, 2));
        assert_eq!(buf, [0xff, 0b111, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00]);

        deser_roundtrip(&mut buf, &DisplayThrottle::new(1, false, true, 2));
        assert_eq!(buf, [0x01, 0b100, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00]);

        deser_roundtrip(&mut buf, &DisplayThrottle::new(256, false, true, 2));
        assert_eq!(buf, [0x00, 0b101, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_display_charge_history_request() {
        let mut buf = [0u8; 2];
        deser_roundtrip(&mut buf, &DisplayChargeHistoryRequest::new());
        assert_eq!(buf, [0x01, 0xFE]);
    }
}
