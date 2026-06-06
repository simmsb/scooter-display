#![allow(unused)]

use core::str::FromStr as _;

use heapless::LenType;

#[derive(deku::DekuRead, deku::DekuWrite, deku::DekuSize, defmt::Format)]
#[deku(id_type = "u16", endian = "little")]
#[repr(u16)]
pub enum Endpoint {
    // 0. Unknown, has no callbacks in firmware
    Unknown0 = 0,
    // 1. Unknown, has no callbacks in firmware
    Unknown1 = 1,
    // 2. Returns bit 29 of some bit flags stored on the display. Seems to be the 'bluetooth enabled' bit
    BluetoothEnabledBit = 2,
    // 3. Used to set connection state
    SetConnectionState = 3,
    // 4. Unknown, tx callback is noo
    Unknown4 = 4,
    // 5. Device information (VIN, device name, firmware versions
    DeviceInformation = 5,
    // 6. System status (battery level, power output, etc
    SystemStatus = 6,
    // 7. System status (unknown fields
    SystemStatusUnknown = 7,
    // 8. 'operation' command handle
    OperationHandle = 8,
    // 9. Device state (power/locked/lights/charging/range
    DeviceState = 9,
    // 10. Odometer
    Odometer = 10,
    // 11. 'settings' command handler
    SettingsHandler = 11,
    // 12. Current 'settings' report
    SettingsReport = 12,
    // 13. Set customer name
    SetCustomerName = 13,
    // 14. Report customer name
    ReportCustomerName = 14,
    // 15. Unknown. Seems to do nothing
    Unknown15 = 15,
    // 16. Might be current speed
    CurrentSpeed = 16,
    // 17. Reports charge history
    ChargeHistory = 17,
    // 18. Reports failure code
    FailureCode = 18,
    // 19. Reports battery percent and total time active
    BatteryAndActiveTime = 19,
    // 20. Reports time spent in each drive mode
    DriveModeHistory = 20,
    // 21. Unknown, does nothing
    Unknown21 = 21,
    // 22. Unknown, does nothing
    Unknown22 = 22,
    // 23. Might be update progress
    UpdateProgress = 23,
    // 24. Paired or other connected status
    ConnectedStatus = 24,
    // 25. Initiates bluetooth update
    InitiateBluetoothUpdate = 25,
    // 26. Reports bluetooth update state
    BluetoothUpdateState = 26,
}

pub trait EndpointValue {
    fn endpoint_value() -> Endpoint;
}

#[derive(deku::DekuRead, defmt::Format)]
#[deku(id_type = "u16", id_endian = "little", magic = b"\0")]
#[repr(u16)]
pub enum Command {
    #[deku(id = 0x00)]
    Unknown0(Unknown0Command),
    #[deku(id = 0x01)]
    Unknown1(Unknown1Command),
    #[deku(id = 0x02)]
    BluetoothEnabledBit(BluetoothEnabledBitCommand),
    #[deku(id = 0x03)]
    SetConnectionState(SetConnectionStateCommand),
    #[deku(id = 0x04)]
    Unknown4(Unknown4Command),
    #[deku(id = 0x05)]
    DeviceInformation(DeviceInformationCommand),
    #[deku(id = 0x06)]
    SystemStatus(SystemStatusCommand),
    #[deku(id = 0x07)]
    SystemStatusUnknown(SystemStatusUnknownCommand),
    #[deku(id = 0x08)]
    OperationCommand(OperationHandleCommand),
    #[deku(id = 0x09)]
    DeviceState(DeviceStateCommand),
    #[deku(id = 0x0a)]
    Odometer(OdometerCommand),
    #[deku(id = 0x0b)]
    SettingsHandler(SettingsHandlerCommand),
    #[deku(id = 0x0c)]
    SettingsReport(SettingsReportCommand),
    #[deku(id = 0x0d)]
    SetCustomerName(SetCustomerNameCommand),
    #[deku(id = 0x0e)]
    ReportCustomerName(ReportCustomerNameCommand),
    #[deku(id = 0x0f)]
    Unknown15(Unknown15Command),
    #[deku(id = 0x10)]
    CurrentSpeed(CurrentSpeedCommand),
    #[deku(id = 0x11)]
    ChargeHistory(ChargeHistoryCommand),
    #[deku(id = 0x12)]
    FailureCode(FailureCodeCommand),
    #[deku(id = 0x13)]
    BatteryAndActiveTime(BatteryAndActiveTimeCommand),
    #[deku(id = 0x14)]
    DriveModeHistory(DriveModeHistoryCommand),
    #[deku(id = 0x15)]
    Unknown21(Unknown21Command),
    #[deku(id = 0x16)]
    Unknown22(Unknown22Command),
    #[deku(id = 0x17)]
    UpdateProgress(UpdateProgressCommand),
    #[deku(id = 0x18)]
    ConnectedStatus(ConnectedStatusCommand),
    #[deku(id = 0x19)]
    InitiateBluetoothUpdate(InitiateBluetoothUpdateCommand),
}

impl Command {
    pub fn endpoint_value(&self) -> Endpoint {
        match self {
            Command::Unknown0(_) => Endpoint::Unknown0,
            Command::Unknown1(_) => Endpoint::Unknown1,
            Command::BluetoothEnabledBit(_) => Endpoint::BluetoothEnabledBit,
            Command::SetConnectionState(_) => Endpoint::SetConnectionState,
            Command::Unknown4(_) => Endpoint::Unknown4,
            Command::DeviceInformation(_) => Endpoint::DeviceInformation,
            Command::SystemStatus(_) => Endpoint::SystemStatus,
            Command::SystemStatusUnknown(_) => Endpoint::SystemStatusUnknown,
            Command::OperationCommand(_) => Endpoint::OperationHandle,
            Command::DeviceState(_) => Endpoint::DeviceState,
            Command::Odometer(_) => Endpoint::Odometer,
            Command::SettingsHandler(_) => Endpoint::SettingsHandler,
            Command::SettingsReport(_) => Endpoint::SettingsReport,
            Command::SetCustomerName(_) => Endpoint::SetCustomerName,
            Command::ReportCustomerName(_) => Endpoint::ReportCustomerName,
            Command::Unknown15(_) => Endpoint::Unknown15,
            Command::CurrentSpeed(_) => Endpoint::CurrentSpeed,
            Command::ChargeHistory(_) => Endpoint::ChargeHistory,
            Command::FailureCode(_) => Endpoint::FailureCode,
            Command::BatteryAndActiveTime(_) => Endpoint::BatteryAndActiveTime,
            Command::DriveModeHistory(_) => Endpoint::DriveModeHistory,
            Command::Unknown21(_) => Endpoint::Unknown21,
            Command::Unknown22(_) => Endpoint::Unknown22,
            Command::UpdateProgress(_) => Endpoint::UpdateProgress,
            Command::ConnectedStatus(_) => Endpoint::ConnectedStatus,
            Command::InitiateBluetoothUpdate(_) => Endpoint::InitiateBluetoothUpdate,
        }
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct Unknown0Command;
impl EndpointValue for Unknown0Command {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown0
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct Unknown1Command;
impl EndpointValue for Unknown1Command {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown1
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct BluetoothEnabledBitCommand;
impl EndpointValue for BluetoothEnabledBitCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::BluetoothEnabledBit
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct SetConnectionStateCommand {
    connected: bool,
}

impl EndpointValue for SetConnectionStateCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::SetConnectionState
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct Unknown4Command;
impl EndpointValue for Unknown4Command {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown4
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct DeviceInformationCommand;
impl EndpointValue for DeviceInformationCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceInformation
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct SystemStatusCommand;
impl EndpointValue for SystemStatusCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::SystemStatus
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct SystemStatusUnknownCommand;
impl EndpointValue for SystemStatusUnknownCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::SystemStatusUnknown
    }
}

// private static final byte SET_POWERED_ON = 1;
// private static final byte SET_LOCKED = 2;
// private static final byte SET_LIGHTS = 3;
// private static final byte SET_FREE_DISPLAY_TEXT = 4;
// private static final byte SET_DRIVING_MODE = 6;
// private static final byte SET_TEMPERATURE_WARNING = 7;
// private static final byte SET_RANGE_CORRECTION_FACTOR = 8;
// private static final byte RESET_ODO_TRIP = 9;
// private static final byte RESET_ODO_DRIVING_MODE = 10;
// private static final byte RESET_ODO_TOTAL = 11;
// private static final byte CLEAR_FREE_DISPLAY_TEXT = 12;
// private static final byte SHOW_FREE_DISPLAY_TEXT = 13;
// private static final int FREE_TEXT_FRAME_SIZE = 18;

#[derive(defmt::Format, deku::DekuRead)]
#[deku(id_type = "u8")]
pub enum OperationHandleCommand {
    #[deku(id = 0x1)]
    SetPoweredOn,

    #[deku(id = 0x2)]
    SetLocked(bool),

    #[deku(id = 0x3)]
    SetLightsOn(bool),

    #[deku(id = 0x4)]
    SetFreeDisplayText {
        slot: u8,
        buf: BluetoothString<12, u8>,
    },

    #[deku(id = 0x6)]
    SetDrivingMode(u8),

    #[deku(id = 0x7)]
    SetTemperatureWarning,

    #[deku(id = 0x8)]
    SetRangeCorrectionFactor(u8),

    #[deku(id = 0x9)]
    ResetOdoTrip,

    #[deku(id = 0xA)]
    ResetOdoForMode(u8),

    #[deku(id = 0xB)]
    ResetOdoTotal,

    #[deku(id = 0xC)]
    ClearFreeText(u8),

    #[deku(id = 0x13)]
    ShowFreeDisplayText,

    #[deku(id = 0x12)]
    FreeTextFrameSize(u8),

    #[deku(id = 0x50)]
    SyncRTC {
        #[deku(endian = "little")]
        timestamp_millis: i64,
    },
}

impl EndpointValue for OperationHandleCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::OperationHandle
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct DeviceStateCommand;
impl EndpointValue for DeviceStateCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceState
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct OdometerCommand;
impl EndpointValue for OdometerCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::Odometer
    }
}

// private static final byte SET_LANGUAGE = 1;
// private static final byte SET_DISPLAY_BRIGHTNESS = 2;
// private static final byte SET_OPTION_UNLOCK_SCOOTER_DISPLAY = 3;
// private static final byte SET_UNLOCK_CODE_DISPLAY = 4;
// private static final byte SET_ACTIVATION = 5;
// private static final byte SET_ACTIVATION_CODE = 6;
// private static final byte SET_SPEED_LIMIT_ENABLED = 7;
// private static final byte SET_SPEED_LIMIT = 8;
// private static final byte SET_SPEED_UNIT = 9;
// private static final byte SET_BLUETOOTH_ALWAYS_ON = 10;
// private static final byte SET_SERIAL_NUMBER = 12;
// private static final byte SET_AUTOMATIC_HEADLIGHTS = 13;

#[derive(defmt::Format, deku::DekuRead)]
#[deku(id_type = "u8")]
pub enum SettingsHandlerCommand {
    #[deku(id = 0x1)]
    SetLanguage(u8),

    #[deku(id = 0x2)]
    SetBrightness(u8),

    #[deku(id = 0x3)]
    UnlockDisplay(bool),

    #[deku(id = 0x4)]
    SetUnlockCode(BluetoothString<4, u8>),

    #[deku(id = 0x5)]
    SetActivation(bool),

    #[deku(id = 0x6)]
    SetActivationCode(BluetoothString<4, u8>),

    #[deku(id = 0x7)]
    SetSpeedLimitEnabled(bool),

    #[deku(id = 0x8)]
    SetSpeedLimit(u8),

    #[deku(id = 0x9)]
    SetSpeedUnit(u8),

    #[deku(id = 0xA)]
    SetBluetoothAlwaysOn(bool),

    #[deku(id = 0xC)]
    SetVin(BluetoothString<15, u8>),

    #[deku(id = 0xD)]
    SetAutomaticHeadlights(bool),

    #[deku(id = 0xE)]
    SetActiveNFCKey(u8),
}

impl EndpointValue for SettingsHandlerCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::SettingsHandler
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct SettingsReportCommand;
impl EndpointValue for SettingsReportCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::SettingsReport
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct SetCustomerNameCommand;
impl EndpointValue for SetCustomerNameCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::SetCustomerName
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct ReportCustomerNameCommand;
impl EndpointValue for ReportCustomerNameCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::ReportCustomerName
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct Unknown15Command;
impl EndpointValue for Unknown15Command {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown15
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct CurrentSpeedCommand;
impl EndpointValue for CurrentSpeedCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::CurrentSpeed
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct ChargeHistoryCommand;
impl EndpointValue for ChargeHistoryCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::ChargeHistory
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct FailureCodeCommand;
impl EndpointValue for FailureCodeCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::FailureCode
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct BatteryAndActiveTimeCommand;
impl EndpointValue for BatteryAndActiveTimeCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::BatteryAndActiveTime
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct DriveModeHistoryCommand;
impl EndpointValue for DriveModeHistoryCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::DriveModeHistory
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct Unknown21Command;
impl EndpointValue for Unknown21Command {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown21
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct Unknown22Command;
impl EndpointValue for Unknown22Command {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown22
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct UpdateProgressCommand;
impl EndpointValue for UpdateProgressCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::UpdateProgress
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct ConnectedStatusCommand;
impl EndpointValue for ConnectedStatusCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::ConnectedStatus
    }
}

#[derive(defmt::Format, deku::DekuRead)]
pub struct InitiateBluetoothUpdateCommand;
impl EndpointValue for InitiateBluetoothUpdateCommand {
    fn endpoint_value() -> Endpoint {
        Endpoint::InitiateBluetoothUpdate
    }
}

#[derive(deku::DekuWrite, deku::DekuSize, defmt::Format)]
#[deku(id_type = "u16", id_endian = "little", magic = b"\0")]
#[repr(u16)]
pub enum Response {
    #[deku(id = 0x00)]
    Unknown0(Unknown0Response),
    #[deku(id = 0x01)]
    Unknown1(Unknown1Response),
    #[deku(id = 0x02)]
    BluetoothEnabledBit(BluetoothEnabledBitResponse),
    #[deku(id = 0x03)]
    SetConnectionState(SetConnectionStateResponse),
    #[deku(id = 0x04)]
    Unknown4(Unknown4Response),
    #[deku(id = 0x05)]
    DeviceInformation(DeviceInformationResponse),
    #[deku(id = 0x06)]
    SystemStatus(SystemStatusResponse),
    #[deku(id = 0x07)]
    SystemStatusUnknown(SystemStatusUnknownResponse),
    #[deku(id = 0x08)]
    OperationCommand(OperationCommandResponse),
    #[deku(id = 0x09)]
    DeviceState(DeviceStateResponse),
    #[deku(id = 0x0a)]
    Odometer(OdometerResponse),
    #[deku(id = 0x0b)]
    SettingsHandler(SettingsHandlerResponse),
    #[deku(id = 0x0c)]
    SettingsReport(SettingsReportResponse),
    #[deku(id = 0x0d)]
    SetCustomerName(SetCustomerNameResponse),
    #[deku(id = 0x0e)]
    ReportCustomerName(ReportCustomerNameResponse),
    #[deku(id = 0x0f)]
    Unknown15(Unknown15Response),
    #[deku(id = 0x10)]
    CurrentSpeed(CurrentSpeedResponse),
    #[deku(id = 0x11)]
    ChargeHistory(ChargeHistoryResponse),
    #[deku(id = 0x12)]
    FailureCode(FailureCodeResponse),
    #[deku(id = 0x13)]
    BatteryAndActiveTime(BatteryAndActiveTimeResponse),
    #[deku(id = 0x14)]
    DriveModeHistory(DriveModeHistoryResponse),
    #[deku(id = 0x15)]
    Unknown21(Unknown21Response),
    #[deku(id = 0x16)]
    Unknown22(Unknown22Response),
    #[deku(id = 0x17)]
    UpdateProgress(SomeBTStatusResponse),
    #[deku(id = 0x18)]
    ConnectedStatus(ConnectedStatusResponse),
    #[deku(id = 0x19)]
    InitiateBluetoothUpdate(InitiateBluetoothUpdateResponse),
}

impl Response {
    pub fn endpoint_value(&self) -> Endpoint {
        match self {
            Response::Unknown0(_) => Endpoint::Unknown0,
            Response::Unknown1(_) => Endpoint::Unknown1,
            Response::BluetoothEnabledBit(_) => Endpoint::BluetoothEnabledBit,
            Response::SetConnectionState(_) => Endpoint::SetConnectionState,
            Response::Unknown4(_) => Endpoint::Unknown4,
            Response::DeviceInformation(_) => Endpoint::DeviceInformation,
            Response::SystemStatus(_) => Endpoint::SystemStatus,
            Response::SystemStatusUnknown(_) => Endpoint::SystemStatusUnknown,
            Response::OperationCommand(_) => Endpoint::OperationHandle,
            Response::DeviceState(_) => Endpoint::DeviceState,
            Response::Odometer(_) => Endpoint::Odometer,
            Response::SettingsHandler(_) => Endpoint::SettingsHandler,
            Response::SettingsReport(_) => Endpoint::SettingsReport,
            Response::SetCustomerName(_) => Endpoint::SetCustomerName,
            Response::ReportCustomerName(_) => Endpoint::ReportCustomerName,
            Response::Unknown15(_) => Endpoint::Unknown15,
            Response::CurrentSpeed(_) => Endpoint::CurrentSpeed,
            Response::ChargeHistory(_) => Endpoint::ChargeHistory,
            Response::FailureCode(_) => Endpoint::FailureCode,
            Response::BatteryAndActiveTime(_) => Endpoint::BatteryAndActiveTime,
            Response::DriveModeHistory(_) => Endpoint::DriveModeHistory,
            Response::Unknown21(_) => Endpoint::Unknown21,
            Response::Unknown22(_) => Endpoint::Unknown22,
            Response::UpdateProgress(_) => Endpoint::UpdateProgress,
            Response::ConnectedStatus(_) => Endpoint::ConnectedStatus,
            Response::InitiateBluetoothUpdate(_) => Endpoint::InitiateBluetoothUpdate,
        }
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct Unknown0Response;
impl EndpointValue for Unknown0Response {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown0
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct Unknown1Response;
impl EndpointValue for Unknown1Response {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown1
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct BluetoothEnabledBitResponse;
impl EndpointValue for BluetoothEnabledBitResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::BluetoothEnabledBit
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SetConnectionStateResponse;
impl EndpointValue for SetConnectionStateResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::SetConnectionState
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct Unknown4Response;
impl EndpointValue for Unknown4Response {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown4
    }
}

#[cfg_attr(test, derive(deku::DekuRead, PartialEq, Debug))]
#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct DeviceInformationResponse {
    pub vin: BluetoothString<20, u8>,
    pub model: BluetoothString<20, u8>,
    pub hardware_version: BluetoothString<20, u8>,
    pub controller_firmware_version: BluetoothString<20, u8>,
    pub display_firmware_version: BluetoothString<20, u8>,
}

impl EndpointValue for DeviceInformationResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceInformation
    }
}

//  bVar3 = BATTERY_PCT;
// SYSTEM_STATUS_BT_BUF[0] = bVar3;
//
//  bVar1 = can_1029_buf;
// SYSTEM_STATUS_BT_BUF[1] = bVar1 / 10 + 0x30;
// SYSTEM_STATUS_BT_BUF[2] = 0x2e;
// bVar1 = can_1029_buf;
// SYSTEM_STATUS_BT_BUF[3] = bVar1 % 10 + 0x30;
// SYSTEM_STATUS_BT_BUF[4] = 0x2e;
// SYSTEM_STATUS_BT_BUF[5] = 0x30;
// SYSTEM_STATUS_BT_BUF[6] = 0;
// return;
//
// SYSTEM_STATUS_BT_BUF[0x15] = 0;
// bVar3 = WRITTEN_BY_CAN_1857._0_1_;
// SYSTEM_STATUS_BT_BUF[0x16] = bVar3;
// iVar2 = absolute_soh;
// SYSTEM_STATUS_BT_BUF[0x17] = (byte)((uint)(iVar2 * 0x30) / 1000 >> 8);
// iVar2 = absolute_soh;
// SYSTEM_STATUS_BT_BUF[0x18] = (byte)((uint)(iVar2 * 0x30) / 1000);
// uVar4 = Ram20000153;
// SYSTEM_STATUS_BT_BUF[0x19] = (byte)((ushort)uVar4 >> 8);
// bVar3 = DAT_20000150._3_1_;
// SYSTEM_STATUS_BT_BUF[0x1a] = bVar3;
// uVar1 = Ram20000135;
// SYSTEM_STATUS_BT_BUF[0x1b] = (byte)(uVar1 / 100 >> 8);
// uVar1 = Ram20000135;
// SYSTEM_STATUS_BT_BUF[0x1c] = (byte)(uVar1 / 100);
// iVar2 = CURRENT_POWER_OUTPUT;
// SYSTEM_STATUS_BT_BUF[0x1d] = (byte)((uint)(iVar2 / 100) >> 8);
// iVar2 = CURRENT_POWER_OUTPUT;
// SYSTEM_STATUS_BT_BUF[0x1e] = (byte)(iVar2 / 100);
// bluetooth_headers[6].buf_len = 0x26;
// bluetooth_headers[6].buffer = SYSTEM_STATUS_BT_BUF;

#[cfg_attr(test, derive(deku::DekuRead, PartialEq, Debug))]
#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SystemStatusResponse {
    pub battery_pct: u8,

    pub some_pct_str: BluetoothString<6, u8>,

    #[deku(pad_bytes_before = "13")]
    pub unknown: u8,

    #[deku(endian = "big")]
    pub absolute_soh: u16,

    #[deku(endian = "big")]
    pub charge_state: u16,

    #[deku(endian = "big")]
    pub unknown_2: u16,

    #[deku(endian = "big")]
    pub voltage: u16,

    #[deku(endian = "big")]
    pub current: u16,
}

impl EndpointValue for SystemStatusResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::SystemStatus
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SystemStatusUnknownResponse;
impl EndpointValue for SystemStatusUnknownResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::SystemStatusUnknown
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct OperationCommandResponse;
impl EndpointValue for OperationCommandResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::OperationHandle
    }
}

// boolean zFlag = PayloadUtilsKt.flag(bytes[0], 0);
// boolean zFlag2 = PayloadUtilsKt.flag(bytes[0], 1);
// boolean zFlag3 = PayloadUtilsKt.flag(bytes[0], 2);
// boolean zFlag4 = PayloadUtilsKt.flag(bytes[0], 3);
// boolean zFlag5 = PayloadUtilsKt.flag(bytes[0], 4);
// boolean zFlag6 = PayloadUtilsKt.flag(bytes[0], 5);
// float fUInt16 = PayloadUtilsKt.uInt16(bytes, 1) / 10.0f;
// int iUInt8 = PayloadUtilsKt.uInt8(bytes, 3);
// float fUInt162 = PayloadUtilsKt.uInt16(bytes, 4) / 10.0f;
// float fUInt163 = PayloadUtilsKt.uInt16(bytes, 6) / 10.0f;
// float fUInt164 = PayloadUtilsKt.uInt16(bytes, 8) / 10.0f;
// int iUInt82 = PayloadUtilsKt.uInt8(bytes, 10);
// int iUInt83 = PayloadUtilsKt.uInt8(bytes, 11);
// DrivingMode drivingModeFromByte = DrivingMode.INSTANCE.fromByte(bytes[12]);
// ErrorCode errorCodeFromByte = ErrorCode.INSTANCE.fromByte(bytes[13]);
// FindMyStatus.Companion companion = FindMyStatus.INSTANCE;
// Byte orNull = ArraysKt.getOrNull(bytes, 14);
//
// this.poweredOn = poweredOn;
// this.locked = z;
// this.lightsOn = z2;
// this.charging = z3;
// this.temperatureLow = z4;
// this.temperatureHigh = z5;
// this.speed = f;
// this.powerOutput = i;
// this.ecoModeRange = f2;
// this.tourModeRange = f3;
// this.sportModeRange = f4;
// this.rangeFactor = i2;
// this.throttle = i3;
// this.drivingMode = drivingMode;
// this.errorCode = errorCode;
// this.findMyStatus = findMyStatus;

// 01000000000000000000640001FEFF

#[cfg_attr(test, derive(deku::DekuRead, PartialEq, Debug))]
#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct OriginalDeviceStateResponse {
    #[deku(bits = 1, pad_bits_before = "2")]
    pub temperature_high: bool,

    #[deku(bits = 1)]
    pub temperature_low: bool,

    #[deku(bits = 1)]
    pub charging: bool,

    #[deku(bits = 1)]
    pub lights_on: bool,

    #[deku(bits = 1)]
    pub locked: bool,

    #[deku(bits = 1)]
    pub powered_on: bool,

    #[deku(endian = "big")]
    pub speed: u16,
    pub power_output: u8,

    #[deku(endian = "big")]
    pub eco_range: u16,

    #[deku(endian = "big")]
    pub tour_range: u16,

    #[deku(endian = "big")]
    pub sport_range: u16,

    #[deku(endian = "big")]
    pub range_factor: u8,

    pub throttle: u8,
    pub driving_mode: u8,

    pub error_code: u8,
    pub find_my_status: u8,
}

impl EndpointValue for OriginalDeviceStateResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceState
    }
}

#[cfg_attr(test, derive(deku::DekuRead, PartialEq, Debug))]
#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
#[deku(endian = "big")]
pub struct DeviceStateResponse {
    // must be 15 bytes
    #[deku(bits = 1, pad_bits_before = "5")]
    pub charging: bool,

    #[deku(bits = 1)]
    pub lights_on: bool,

    #[deku(bits = 1)]
    pub locked: bool,

    pub speed: u16,
    pub power_output: u16,
    pub range: u16,
    pub throttle: u8,
    pub driving_mode: u8,
    pub odo: u16,
    pub temp: u8,
    pub ambient: u8,
    pub soc: u16,
}

impl EndpointValue for DeviceStateResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceState
    }
}

// BLUETOOTH_17_BUF[0] = (byte)(TIME_SPENT_TOTAL / 100 >> 8);
// BLUETOOTH_17_BUF[1] = (byte)(TIME_SPENT_TOTAL / 100);
// BLUETOOTH_17_BUF[2] = (byte)(TIME_SPENT_TOTAL / 1000 >> 0x10);
// BLUETOOTH_17_BUF[3] = (byte)(TIME_SPENT_TOTAL / 1000 >> 8);
// BLUETOOTH_17_BUF[4] = (byte)(TIME_SPENT_TOTAL / 1000);
// BLUETOOTH_17_BUF[5] = (byte)(TIME_SPENT_IN_ECO / 1000 >> 0x10);
// BLUETOOTH_17_BUF[6] = (byte)(TIME_SPENT_IN_ECO / 1000 >> 8);
// BLUETOOTH_17_BUF[7] = (byte)(TIME_SPENT_IN_ECO / 1000);
// BLUETOOTH_17_BUF[8] = (byte)(TIME_SPENT_IN_DRIVE / 1000 >> 0x10);
// BLUETOOTH_17_BUF[9] = (byte)(TIME_SPENT_IN_DRIVE / 1000 >> 8);
// BLUETOOTH_17_BUF[10] = (byte)(TIME_SPENT_IN_DRIVE / 1000);
// BLUETOOTH_17_BUF[0xb] = (byte)(TIME_SPENT_IN_SPORT / 1000 >> 0x10);
// BLUETOOTH_17_BUF[0xc] = (byte)(TIME_SPENT_IN_SPORT / 1000 >> 8);
// BLUETOOTH_17_BUF[0xd] = (byte)(TIME_SPENT_IN_SPORT / 1000);

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct OdometerResponse;

impl EndpointValue for OdometerResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::Odometer
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SettingsHandlerResponse;
impl EndpointValue for SettingsHandlerResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::SettingsHandler
    }
}

// Intrinsics.checkNotNullParameter(bytes, "bytes");
// boolean zFlag = PayloadUtilsKt.flag(bytes[0], 0); // activated
// boolean zFlag2 = PayloadUtilsKt.flag(bytes[0], 1); // display lock
// boolean zFlag3 = PayloadUtilsKt.flag(bytes[0], 2); // speed limit
// boolean zFlag4 = PayloadUtilsKt.flag(bytes[0], 3);  // bluetooth always on
// Language languageFromByte = Language.INSTANCE.fromByte(bytes[1]);
// byte b = bytes[2];
// String strDecodeToString = StringsKt.decodeToString(ArraysKt.copyOfRange(bytes, 3, 7));
// String strDecodeToString2 = StringsKt.decodeToString(ArraysKt.copyOfRange(bytes, 7, 11));
// SpeedLimit speedLimitFromByte = SpeedLimit.INSTANCE.fromByte(bytes[11]);
// SpeedUnit speedUnitFromByte = SpeedUnit.INSTANCE.fromByte(bytes[13]);
// AutomaticHeadlights.Companion companion = AutomaticHeadlights.INSTANCE;
// Byte orNull = ArraysKt.getOrNull(bytes, 14);
// return new Settings(zFlag, zFlag2, zFlag3, zFlag4, languageFromByte, b, strDecodeToString, strDecodeToString2, speedLimitFromByte, speedUnitFromByte, companion.fromByte(orNull != null ? orNull.byteValue() : (byte) -1));

//    public Settings(boolean z, boolean z2, boolean z3, boolean z4, Language language, int i, String unlockCodeDisplay, String activationCode, SpeedLimit speedLimit, SpeedUnit speedUnit, AutomaticHeadlights automaticHeadlights) {
//         Intrinsics.checkNotNullParameter(language, "language");
//         Intrinsics.checkNotNullParameter(unlockCodeDisplay, "unlockCodeDisplay");
//         Intrinsics.checkNotNullParameter(activationCode, "activationCode");
//         Intrinsics.checkNotNullParameter(speedLimit, "speedLimit");
//         Intrinsics.checkNotNullParameter(speedUnit, "speedUnit");
//         Intrinsics.checkNotNullParameter(automaticHeadlights, "automaticHeadlights");
//         this.isActivated = z;
//         this.displayLockEnabled = z2;
//         this.speedLimitEnabled = z3;
//         this.bluetoothAlwaysOn = z4;
//         this.language = language;
//         this.brightness = i;
//         this.unlockCodeDisplay = unlockCodeDisplay;
//         this.activationCode = activationCode;
//         this.speedLimit = speedLimit;
//         this.speedUnit = speedUnit;
//         this.automaticHeadlights = automaticHeadlights;
//     }

#[cfg_attr(test, derive(deku::DekuRead, PartialEq, Debug))]
#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SettingsReportResponse {
    #[deku(bits = 1, pad_bits_before = "4")]
    pub bluetooth_always_on: bool,
    #[deku(bits = 1)]
    pub speed_limit_enabled: bool,
    #[deku(bits = 1)]
    pub display_lock: bool,
    #[deku(bits = 1)]
    pub activated: bool,

    pub language: u8,
    pub brightness: u8,

    pub unlock_code: BluetoothString<4, u8>,
    pub activation_code: BluetoothString<4, u8>,

    pub speed_limit: u8,
    pub unknown: u8,
    pub speed_unit: u8,
    pub headlights_config: u8,
    pub nfc_key_presence: u8,
    pub active_nfc_key: u32,
}

impl SettingsReportResponse {
    pub fn new(
        activated: bool,
        display_lock: bool,
        speed_limit_enabled: bool,
        bluetooth_always_on: bool,
        language: u8,
        brightness: u8,
        unlock_code: BluetoothString<4, u8>,
        activation_code: BluetoothString<4, u8>,
        speed_limit: u8,
        unknown: u8,
        speed_unit: u8,
        headlights_config: u8,
        nfc_key_presence: u8,
        active_nfc_key: u32,
    ) -> Self {
        Self {
            activated,
            display_lock,
            speed_limit_enabled,
            bluetooth_always_on,
            language,
            brightness,
            unlock_code,
            activation_code,
            speed_limit,
            unknown,
            speed_unit,
            headlights_config,
            nfc_key_presence,
            active_nfc_key,
        }
    }
}

impl EndpointValue for SettingsReportResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::SettingsReport
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SetCustomerNameResponse;
impl EndpointValue for SetCustomerNameResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::SetCustomerName
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct ReportCustomerNameResponse {
    pub name: BluetoothString<20, u8>,
}

impl EndpointValue for ReportCustomerNameResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::ReportCustomerName
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct Unknown15Response;
impl EndpointValue for Unknown15Response {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown15
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct CurrentSpeedResponse;
impl EndpointValue for CurrentSpeedResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::CurrentSpeed
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct ChargeHistoryResponse;
impl EndpointValue for ChargeHistoryResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::ChargeHistory
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct FailureCodeResponse;
impl EndpointValue for FailureCodeResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::FailureCode
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct BatteryAndActiveTimeResponse {
    pub timestamp: u32,

    pub battery_pct: u8,

    #[deku(bytes = 3, endian = "big")]
    pub odometer: u32,
}

impl EndpointValue for BatteryAndActiveTimeResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::BatteryAndActiveTime
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct DriveModeHistoryResponse {
    pub timestamp: u32,

    pub battery_pct: u8,

    #[deku(bytes = 3, endian = "big")]
    pub time_total: u32,

    #[deku(bytes = 3, endian = "big")]
    pub time_total_eco: u32,

    #[deku(bytes = 3, endian = "big")]
    pub time_total_drive: u32,

    #[deku(bytes = 3, endian = "big")]
    pub time_total_sport: u32,
}

impl EndpointValue for DriveModeHistoryResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::DriveModeHistory
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct Unknown21Response;
impl EndpointValue for Unknown21Response {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown21
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct Unknown22Response;
impl EndpointValue for Unknown22Response {
    fn endpoint_value() -> Endpoint {
        Endpoint::Unknown22
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SomeBTStatusResponse {
    pub flag0: bool,
    #[deku(pad_bytes_after = "6")]
    pub flag1: bool,
}

impl EndpointValue for SomeBTStatusResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::UpdateProgress
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct ConnectedStatusResponse;
impl EndpointValue for ConnectedStatusResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::ConnectedStatus
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct InitiateBluetoothUpdateResponse;
impl EndpointValue for InitiateBluetoothUpdateResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::InitiateBluetoothUpdate
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct BluetoothString<const N: usize, LenT: LenType>(pub heapless::String<N, LenT>);

impl<const N: usize, LenT: LenType> BluetoothString<N, LenT> {
    pub fn new(s: &str) -> Self {
        Self(heapless::String::from_str(s).unwrap())
    }
}

impl<const N: usize, LenT: LenType> defmt::Format for BluetoothString<N, LenT> {
    fn format(&self, fmt: defmt::Formatter) {
        self.0.format(fmt);
    }
}

impl<const N: usize, LenT: LenType> deku::DekuWriter for BluetoothString<N, LenT> {
    #[doc = " Write type to bytes"]
    fn to_writer<W: deku::no_std_io::Write + deku::no_std_io::Seek>(
        &self,
        writer: &mut deku::writer::Writer<W>,
        ctx: (),
    ) -> Result<(), deku::DekuError> {
        let remainder = N - self.0.len();
        self.0.as_bytes().to_writer(writer, ctx)?;
        for _ in 0..remainder {
            writer.write_bytes(&[0u8])?;
        }
        Ok(())
    }
}

impl<'a, const N: usize, LenT: LenType> deku::DekuReader<'a> for BluetoothString<N, LenT> {
    fn from_reader_with_ctx<R: deku::no_std_io::Read + deku::no_std_io::Seek>(
        reader: &mut deku::reader::Reader<R>,
        ctx: (),
    ) -> Result<Self, deku::DekuError>
    where
        Self: Sized,
    {
        let s = <[u8; N]>::from_reader_with_ctx(reader, ctx)?;
        let mut s = heapless::Vec::from_array(s);
        if let Some(null_idx) = s.iter().position(|&b| b == 0) {
            s.truncate(null_idx);
        }
        let s = heapless::String::from_utf8(s)
            .map_err(|_| deku::deku_error!(deku::DekuError::Parse, "no nul in string"))?;
        Ok(Self(s))
    }
}

impl<const N: usize, LenT: LenType> deku::DekuSize for BluetoothString<N, LenT> {
    const SIZE_BITS: usize = u8::SIZE_BITS * N;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn settings_report() {
        let serialized = &[
            0x0B, 0x01, 0x04, 0x31, 0x32, 0x33, 0x34, 0x30, 0x30, 0x30, 0x30, 0x00, 0x00, 0x00,
            0x02, 0x01, 0xA5, 0x06, 0x97, 0xC5,
        ];

        let parsed = SettingsReportResponse::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            SettingsReportResponse {
                activated: true,
                display_lock: true,
                speed_limit_enabled: false,
                bluetooth_always_on: true,
                language: 1,
                brightness: 4,
                unlock_code: BluetoothString::new("1234"),
                activation_code: BluetoothString::new("0000"),
                speed_limit: 0,
                unknown: 0,
                speed_unit: 0,
                headlights_config: 2,
                nfc_key_presence: 1,
                active_nfc_key: 0xC59706A5,
            }
        )
    }

    #[test]
    fn device_information() {
        let serialized = &[
            0x57, 0x55, 0x45, 0x47, 0x54, 0x33, 0x31, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
            0x37, 0x38, 0x39, 0x00, 0x00, 0x00, 0x45, 0x47, 0x52, 0x45, 0x54, 0x20, 0x47, 0x54,
            0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x32, 0x2E,
            0x30, 0x2E, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x30, 0x2E, 0x30, 0x2E, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x2E, 0x30, 0x2E,
            0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let parsed = DeviceInformationResponse::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            DeviceInformationResponse {
                vin: BluetoothString::new("WUEGT310123456789"),
                model: BluetoothString::new("EGRET GT1"),
                hardware_version: BluetoothString::new("2.0.0"),
                controller_firmware_version: BluetoothString::new("0.0.0"),
                display_firmware_version: BluetoothString::new("4.0.2"),
            }
        )
    }

    #[test]
    fn device_state_orig() {
        let serialized = &[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x01, 0xFE,
            0xFF,
        ];

        let parsed = OriginalDeviceStateResponse::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            OriginalDeviceStateResponse {
                temperature_high: false,
                temperature_low: false,
                charging: false,
                lights_on: false,
                locked: false,
                powered_on: true,
                speed: 0,
                power_output: 0,
                eco_range: 0,
                tour_range: 0,
                sport_range: 0,
                range_factor: 100,
                throttle: 0,
                driving_mode: 1,
                error_code: 254,
                find_my_status: 255
            }
        )
    }

    #[test]
    fn device_state() {
        let serialized = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x10, 0x11, 0x12, 0x13, 0x14,
            0x15,
        ];

        let parsed = DeviceStateResponse::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            DeviceStateResponse {
                charging: false,
                lights_on: false,
                locked: true,
                speed: 0x0203,
                power_output: 0x0405,
                range: 0x0607,
                throttle: 0x08,
                driving_mode: 0x09,
                odo: 0x1011,
                temp: 0x12,
                ambient: 0x13,
                soc: 0x1415,
            }
        )
    }

    #[test]
    fn system_status() {
        let serialized = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];

        let parsed = SystemStatusResponse::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            SystemStatusResponse {
                battery_pct: 0,
                some_pct_str: BluetoothString::new(""),
                unknown: 0,
                absolute_soh: 0,
                charge_state: 0,
                unknown_2: 0,
                voltage: 0,
                current: 0
            }
        )
    }
}
