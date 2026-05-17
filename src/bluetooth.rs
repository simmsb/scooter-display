// - USART2
// - rx: GPIOA 3 (bit 0x8) (`USART2_RX`)
// - tx: GPIOA 2 (bit 0x4) (`USART2_TX`)
// - 57600 baud

use crate::bluetooth_proto::*;
use at32f4xx_hal::{
    pac::USART2,
    serial::Serial2,
    uart::{Rx, Tx},
};
use deku::DekuContainerRead as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex, zerocopy_channel};
use embassy_time::{Duration, Ticker};
use embedded_io_async::Write as _;
use static_cell::StaticCell;

// `[0x55, length, unknown, handler_idx_low, handler_idx_high, ...[data], crc[0], crc[1]]`

static COMMAND_CHANNEL: StaticCell<
    zerocopy_channel::Channel<'static, blocking_mutex::raw::ThreadModeRawMutex, Command>,
> = StaticCell::new();
static COMMAND_BUF: StaticCell<[Command; 1]> = StaticCell::new();

static EXT_COMMAND_CHANNEL: StaticCell<
    zerocopy_channel::Channel<'static, blocking_mutex::raw::ThreadModeRawMutex, Command>,
> = StaticCell::new();
static EXT_COMMAND_BUF: StaticCell<[Command; 1]> = StaticCell::new();

pub fn start_bluetooth(spawner: Spawner, uart: Serial2) {
    let (bt_tx, bt_rx) = uart.split();

    let buf = COMMAND_BUF.init([const { Command::Unknown0(Unknown0Command) }; _]);
    let (cmd_tx, cmd_rx) = COMMAND_CHANNEL
        .init(zerocopy_channel::Channel::new(buf))
        .split();

    let ext_buf = EXT_COMMAND_BUF.init([const { Command::Unknown0(Unknown0Command) }; _]);
    let (ext_cmd_tx, ext_cmd_rx) = EXT_COMMAND_CHANNEL
        .init(zerocopy_channel::Channel::new(ext_buf))
        .split();

    spawner.spawn(bluetooth_push_task_(ext_cmd_tx).unwrap());
    spawner.spawn(bluetooth_rx(bt_rx, cmd_tx).unwrap());
    spawner.spawn(bluetooth_tx(bt_tx, cmd_rx, ext_cmd_rx).unwrap());
}

#[embassy_executor::task]
async fn bluetooth_rx(
    rx: Rx<USART2, u8>,
    cmd_sender: zerocopy_channel::Sender<'static, blocking_mutex::raw::ThreadModeRawMutex, Command>,
) {
    bluetooth_rx_(rx, cmd_sender).await;
}

async fn bluetooth_rx_(
    mut rx: Rx<USART2, u8>,
    mut cmd_sender: zerocopy_channel::Sender<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
) {
    defmt::info!("bluetooth RX startup");

    loop {
        let mut buf = [0; 48];

        let buf = match crate::framed_reader::read_framed(&mut rx, 0x55, &mut buf).await {
            Ok(buf) => buf,
            Err(e) => {
                defmt::warn!("Framed read error bt rx: {}", e);
                continue;
            }
        };

        let command = match Command::from_bytes((buf, 0)) {
            Ok(((_, _), cmd)) => cmd,
            Err(e) => {
                defmt::warn!(
                    "Command parse error: {} (in: {})",
                    defmt::Debug2Format(&e),
                    buf
                );
                continue;
            }
        };

        // some commands we trigger ourselves which causes bluetooth uc to ack it.
        //
        // for those, ignore them here
        //
        // maybe in the future we could do some reliability stuff?
        let Some(command) = (match command {
            Command::DeviceState(_) => None,
            c => Some(c),
        }) else {
            continue;
        };

        defmt::trace!("Bluetooth RX: {} {}", command, buf);

        let mut slot = cmd_sender.send().await;
        *slot = command;
        slot.send_done();

        // if let Some(b) = <Rx<USART2, u8> as embedded_hal_nb::serial::Read>::read(&mut rx).ok() {
        //     defmt::info!("Bluetooth RX: {}", b);
        // } else {
        //     embassy_time::Timer::after_millis(100).await;
        // }
    }
}

#[embassy_executor::task]
async fn bluetooth_tx(
    tx: Tx<USART2, u8>,
    cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
    ext_cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
) {
    bluetooth_tx_(tx, cmd_receiver, ext_cmd_receiver).await;
}

async fn bluetooth_tx_(
    mut tx: Tx<USART2, u8>,
    mut cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
    mut ext_cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
) {
    defmt::info!("Bluetooth TX startup");

    let mut buf = [0u8; crate::framed_reader::buffer_size_for_type::<Response>()];

    loop {
        let slot = match embassy_futures::select::select(
            cmd_receiver.receive(),
            ext_cmd_receiver.receive(),
        )
        .await
        {
            embassy_futures::select::Either::First(a) => a,
            embassy_futures::select::Either::Second(a) => a,
        };

        let resp = match &*slot {
            Command::Unknown0(_) => None,
            Command::Unknown1(_) => None,
            Command::BluetoothEnabledBit(_) => None,
            Command::SetConnectionState(_) => {
                Some(Response::SetConnectionState(SetConnectionStateResponse))
            }
            Command::Unknown4(_) => None,
            Command::DeviceInformation(_) => {
                Some(Response::DeviceInformation(DeviceInformationResponse {
                    vin: BluetoothString::new("12345"),
                    model: BluetoothString::new("Egret GTS"),
                    hardware_version: BluetoothString::new("4.2.0"),
                    controller_firmware_version: BluetoothString::new("1.9.2"),
                    display_firmware_version: BluetoothString::new("3.5.5"),
                }))
            }
            Command::SystemStatus(_) => Some(Response::SystemStatus(SystemStatusResponse {
                battery_pct: 99,
                some_pct_str: BluetoothString::new("1.2.0"),
                unknown: 36,
                absolute_soh: 32,
                charge_state: 48,
                unknown_2: 57,
                voltage: 48,
                current: 16,
            })),
            Command::SystemStatusUnknown(_) => {
                Some(Response::SystemStatusUnknown(SystemStatusUnknownResponse))
            }
            Command::OperationCommand(_) => {
                Some(Response::OperationCommand(OperationCommandResponse))
            }
            Command::DeviceState(_) => Some(Response::DeviceState(DeviceStateResponse {
                temperature_high: false,
                temperature_low: false,
                charging: false,
                lights_on: true,
                locked: false,
                powered_on: true,
                speed: 0,
                power_output: 0,
                eco_range: 999,
                tour_range: 999,
                sport_range: 999,
                range_factor: 100,
                throttle: 0,
                driving_mode: 3,
                error_code: 0,
                find_my_status: 0,
            })),
            Command::Odometer(_) => None,
            Command::SettingsHandler(_) => Some(Response::SettingsHandler(SettingsHandlerResponse)),
            Command::SettingsReport(_) => Some(Response::SettingsReport(SettingsReportResponse {
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
            })),
            Command::SetCustomerName(_) => None,
            Command::ReportCustomerName(_) => {
                Some(Response::ReportCustomerName(ReportCustomerNameResponse {
                    name: BluetoothString("Hello".parse().unwrap()),
                }))
            }
            Command::Unknown15(_) => None,
            Command::CurrentSpeed(_) => None,
            Command::ChargeHistory(_) => None,
            Command::FailureCode(_) => None,
            Command::BatteryAndActiveTime(_) => Some(Response::BatteryAndActiveTime(
                BatteryAndActiveTimeResponse {
                    timestamp: 123456,
                    battery_pct: 99,
                    time_spent_total: 12345678,
                },
            )),
            Command::DriveModeHistory(_) => {
                Some(Response::DriveModeHistory(DriveModeHistoryResponse {
                    timestamp: 123456,
                    battery_pct: 99,
                    time_total: 69696969,
                    time_total_eco: 0,
                    time_total_drive: 0,
                    time_total_sport: 696942069,
                }))
            }
            Command::Unknown21(_) => None,
            Command::Unknown22(_) => None,
            Command::UpdateProgress(_) => None,
            Command::ConnectedStatus(_) => Some(Response::ConnectedStatus(ConnectedStatusResponse)),
            Command::InitiateBluetoothUpdate(_) => None,
        };

        slot.receive_done();

        if let Some(resp) = resp {
            let to_send = match crate::framed_reader::assemble_framed_deku(&mut buf, 0xaa, &resp) {
                Err(e) => {
                    defmt::error!("Error writing bt response frame: {}", e);

                    continue;
                }
                Ok(to_send) => to_send,
            };

            defmt::trace!("BT Response: {} {}", resp, to_send);

            let _ = tx.write_all(to_send).await;
        }
    }
}

#[embassy_executor::task]
async fn bluetooth_push_task_(
    cmd_sender: zerocopy_channel::Sender<'static, blocking_mutex::raw::ThreadModeRawMutex, Command>,
) {
    bluetooth_push_task(cmd_sender).await;
}

async fn bluetooth_push_task(
    mut cmd_sender: zerocopy_channel::Sender<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
) {
    let device_state_ticker = async {
        // make this more frequent in real life
        let mut ticker = Ticker::every(Duration::from_secs(10));

        loop {
            ticker.next().await;

            let mut slot = cmd_sender.send().await;
            *slot = Command::DeviceState(DeviceStateCommand);
            slot.send_done();

            let mut slot = cmd_sender.send().await;
            *slot = Command::BatteryAndActiveTime(BatteryAndActiveTimeCommand);
            slot.send_done();

            let mut slot = cmd_sender.send().await;
            *slot = Command::DriveModeHistory(DriveModeHistoryCommand);
            slot.send_done();

            let mut slot = cmd_sender.send().await;
            *slot = Command::SystemStatus(SystemStatusCommand);
            slot.send_done();
        }
    };

    embassy_futures::join::join_array([device_state_ticker]).await;
}
