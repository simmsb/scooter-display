#[derive(deku::DekuRead, deku::DekuWrite, deku::DekuSize, defmt::Format)]
#[deku(id_type = "u16", endian = "big")]
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
#[deku(id_type = "u16", id_endian = "big")]
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
    OperationHandle(OperationHandleCommand),
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
            Command::OperationHandle(_) => Endpoint::OperationHandle,
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
pub struct SetConnectionStateCommand;
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

#[derive(defmt::Format, deku::DekuRead)]
pub struct OperationHandleCommand;
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

#[derive(defmt::Format, deku::DekuRead)]
pub struct SettingsHandlerCommand;
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
#[deku(id_type = "u16", id_endian = "big")]
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
    OperationHandle(OperationHandleResponse),
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
    UpdateProgress(UpdateProgressResponse),
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
            Response::OperationHandle(_) => Endpoint::OperationHandle,
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

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct DeviceInformationResponse;
impl EndpointValue for DeviceInformationResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceInformation
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SystemStatusResponse;
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
pub struct OperationHandleResponse;
impl EndpointValue for OperationHandleResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::OperationHandle
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct DeviceStateResponse;
impl EndpointValue for DeviceStateResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::DeviceState
    }
}

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

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct SettingsReportResponse;
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
pub struct ReportCustomerNameResponse;
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
pub struct BatteryAndActiveTimeResponse;
impl EndpointValue for BatteryAndActiveTimeResponse {
    fn endpoint_value() -> Endpoint {
        Endpoint::BatteryAndActiveTime
    }
}

#[derive(defmt::Format, deku::DekuWrite, deku::DekuSize)]
pub struct DriveModeHistoryResponse;
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
pub struct UpdateProgressResponse;
impl EndpointValue for UpdateProgressResponse {
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
