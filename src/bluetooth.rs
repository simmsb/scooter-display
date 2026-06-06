// - USART2
// - rx: GPIOA 3 (bit 0x8) (`USART2_RX`)
// - tx: GPIOA 2 (bit 0x4) (`USART2_TX`)
// - 57600 baud

use crate::{
    bluetooth_proto::*,
    cfg::{HeadlightMode, SpeedMode, Storable, UnlockCode},
    no_inline_future::NoInlineFutExt as _,
    operation::{self, OPERATION_COMMANDS, OperationCommand},
    pin_digit::PinDigit,
    rtc::get_datetime,
    system_state,
};
use at32f4xx_hal::{
    pac::USART2,
    serial::Serial2,
    uart::{Rx, Tx},
};
use chrono::DateTime;
use deku::DekuContainerRead as _;
use embassy_executor::SendSpawner;
use embassy_sync::{blocking_mutex, zerocopy_channel};
use embassy_time::{Duration, Ticker};
use embedded_io_async::Write as _;
use static_cell::{ConstStaticCell, StaticCell};

// `[0x55, length, unknown, handler_idx_low, handler_idx_high, ...[data], crc[0], crc[1]]`

static COMMAND_CHANNEL: StaticCell<
    zerocopy_channel::Channel<'static, blocking_mutex::raw::CriticalSectionRawMutex, Command>,
> = StaticCell::new();
static COMMAND_BUF: StaticCell<[Command; 1]> = StaticCell::new();

static EXT_COMMAND_CHANNEL: StaticCell<
    zerocopy_channel::Channel<'static, blocking_mutex::raw::CriticalSectionRawMutex, Command>,
> = StaticCell::new();
static EXT_COMMAND_BUF: StaticCell<[Command; 1]> = StaticCell::new();

pub fn start_bluetooth(spawner: SendSpawner, uart: Serial2) {
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
    cmd_sender: zerocopy_channel::Sender<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
) {
    bluetooth_rx_(rx, cmd_sender).await;
}

async fn bluetooth_rx_(
    mut rx: Rx<USART2, u8>,
    mut cmd_sender: zerocopy_channel::Sender<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
) {
    defmt::info!("bluetooth RX startup");

    loop {
        let mut buf = [0; 48];

        let buf = match crate::framed_reader::read_framed(&mut rx, 0x55, &mut buf)
            .no_inline()
            .await
        {
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

        defmt::info!("Bluetooth RX: {} {}", command, buf);

        let mut slot = cmd_sender.send().no_inline().await;
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
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
    ext_cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
) {
    bluetooth_tx_(tx, cmd_receiver, ext_cmd_receiver).await;
}

async fn bluetooth_tx_(
    mut tx: Tx<USART2, u8>,
    mut cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
    mut ext_cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
) {
    defmt::info!("Bluetooth TX startup");

    static BUF: ConstStaticCell<[u8; crate::framed_reader::buffer_size_for_type::<Response>()]> =
        ConstStaticCell::new([0; _]);
    let buf = BUF.take();

    loop {
        let slot = match embassy_futures::select::select(
            cmd_receiver.receive(),
            ext_cmd_receiver.receive(),
        )
        .no_inline()
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
            Command::SystemStatus(_) => system_state::read_state(|s| {
                Some(Response::SystemStatus(SystemStatusResponse {
                    battery_pct: s.battery_info.relative_soc,
                    some_pct_str: BluetoothString::new("1.2.0"),
                    unknown: 36,
                    absolute_soh: s.battery_info.absolute_soh,
                    charge_state: 48,
                    unknown_2: 57,
                    voltage: s.system_voltage.from_battery,
                    current: (s.battery_current.saturating_abs() / 100)
                        .saturating_cast_unsigned()
                        .saturating_truncate(),
                }))
            }),
            Command::SystemStatusUnknown(_) => {
                Some(Response::SystemStatusUnknown(SystemStatusUnknownResponse))
            }
            Command::OperationCommand(cmd) => {
                handle_operation_command(cmd).await;
                Some(Response::OperationCommand(OperationCommandResponse))
            }
            Command::DeviceState(_) => {
                let (lights_on, locked, driving_mode) = operation::read_state(|s| match s {
                    operation::OperationState::Locked(_) => (false, true, 0),
                    operation::OperationState::Active(a) => {
                        (a.headlight_on(), false, a.speed_mode as u8)
                    }
                });

                system_state::read_state(|s| {
                    Some(Response::DeviceState(DeviceStateResponse {
                        charging: s.battery_info.charging,
                        lights_on,
                        locked,
                        speed: s.motor_speed / 10,
                        power_output: s
                            .battery_current
                            .saturating_abs()
                            .saturating_cast_unsigned()
                            .saturating_truncate(),
                        range: s.predicted_range,
                        throttle: s.throttle.for_bluetooth(),
                        driving_mode,
                        odo: s.odometer,
                        temp: (s.battery_info.temperature / 10)
                            .saturating_cast_unsigned()
                            .saturating_truncate(),
                        ambient: s.ambient_light.mapped,
                        soc: s.battery_info.absolute_soc,
                    }))
                })
            }
            Command::Odometer(_) => Some(Response::Odometer(OdometerResponse)),
            Command::SettingsHandler(cmd) => {
                handle_setting_command(cmd).await;
                Some(Response::SettingsHandler(SettingsHandlerResponse))
            }
            Command::SettingsReport(_) => Some(Response::SettingsReport(SettingsReportResponse {
                activated: true,
                display_lock: true,
                speed_limit_enabled: false,
                bluetooth_always_on: true,
                language: 1,
                brightness: 4,
                unlock_code: UnlockCode::maybe_get_stored()
                    .map(|c| c.as_bt_string())
                    .unwrap_or(BluetoothString::new("0000")),
                activation_code: UnlockCode::maybe_get_stored()
                    .map(|c| c.as_bt_string())
                    .unwrap_or(BluetoothString::new("0000")),
                speed_limit: 1,
                unknown: 12,
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
            Command::BatteryAndActiveTime(_) => {
                let timestamp = get_datetime().and_utc().timestamp() as u32;
                let battery_pct = system_state::read_state(|s| s.battery_info.relative_soc);
                let odo = crate::cfg::Odometer::maybe_get_stored()
                    .map(|o| o.km())
                    .unwrap_or(0);
                Some(Response::BatteryAndActiveTime(
                    BatteryAndActiveTimeResponse {
                        timestamp,
                        battery_pct,
                        odometer: odo,
                    },
                ))
            }
            Command::DriveModeHistory(_) => {
                let timestamp = get_datetime().and_utc().timestamp() as u32;
                let odo = crate::cfg::Odometer::maybe_get_stored()
                    .map(|o| o.km())
                    .unwrap_or(0);
                system_state::read_state(|s| {
                    Some(Response::DriveModeHistory(DriveModeHistoryResponse {
                        timestamp,
                        battery_pct: s.battery_info.relative_soc,
                        time_total: odo,
                        time_total_eco: odo,
                        time_total_drive: odo,
                        time_total_sport: odo,
                    }))
                })
            }
            Command::Unknown21(_) => None,
            Command::Unknown22(_) => None,
            Command::UpdateProgress(_) => Some(Response::UpdateProgress(SomeBTStatusResponse {
                flag0: false,
                flag1: true,
            })),
            Command::ConnectedStatus(_) => Some(Response::ConnectedStatus(ConnectedStatusResponse)),
            Command::InitiateBluetoothUpdate(_) => None,
        };

        slot.receive_done();

        if let Some(resp) = resp {
            let to_send = match crate::framed_reader::assemble_framed_deku(buf, 0xaa, &resp) {
                Err(e) => {
                    defmt::error!("Error writing bt response frame: {}", e);

                    continue;
                }
                Ok(to_send) => to_send,
            };

            defmt::trace!("BT Response: {} {}", resp, to_send);

            if let Err(err) = tx.write_all(to_send).no_inline().await {
                defmt::error!("BT Tx err: {}", err);
            }
        }
    }
}

async fn handle_setting_command(cmd: &SettingsHandlerCommand) {
    let op_command = match cmd {
        SettingsHandlerCommand::SetUnlockCode(bluetooth_string) => {
            let mut digits = [PinDigit::D0; 4];

            if bluetooth_string.0.len() != 4 {
                defmt::warn!("BT tried to set pin shorter than 4");
                return;
            }

            for (dst, c) in digits.iter_mut().zip(bluetooth_string.0.chars()) {
                let Some(d) = PinDigit::from_char(c) else {
                    defmt::warn!("BT tried to set pin with invalid char");
                    return;
                };

                *dst = d;
            }

            defmt::info!("Set unlock code to {}", digits);
            UnlockCode::update_stored(UnlockCode { digits });
            None
        }
        _ => None,
    };

    if let Some(op_command) = op_command {
        OPERATION_COMMANDS.send(op_command).no_inline().await;
    }
}

async fn handle_operation_command(cmd: &OperationHandleCommand) {
    let op_command = match cmd {
        OperationHandleCommand::SetPoweredOn => None,
        OperationHandleCommand::SetLocked(locked) => Some(if *locked {
            OperationCommand::Lock
        } else {
            OperationCommand::Unlock
        }),
        OperationHandleCommand::SetLightsOn(on) => {
            Some(OperationCommand::SetHeadlightMode(if *on {
                HeadlightMode::On
            } else {
                HeadlightMode::Off
            }))
        }
        OperationHandleCommand::SetDrivingMode(mode) => {
            Some(OperationCommand::SetSpeedMode(match mode {
                0 => SpeedMode::Walk,
                1 => SpeedMode::Eco,
                2 => SpeedMode::Trip,
                _ => SpeedMode::Sport,
            }))
        }
        OperationHandleCommand::SyncRTC { timestamp_millis } => {
            if let Some(dt) = DateTime::from_timestamp_millis(*timestamp_millis) {
                crate::rtc::set_datetime(dt.naive_utc());
            }

            None
        }
        _ => None,
    };

    if let Some(op_command) = op_command {
        OPERATION_COMMANDS.send(op_command).no_inline().await;
    }
}

#[embassy_executor::task]
async fn bluetooth_push_task_(
    cmd_sender: zerocopy_channel::Sender<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
) {
    bluetooth_push_task(cmd_sender).await;
}

async fn bluetooth_push_task(
    mut cmd_sender: zerocopy_channel::Sender<
        'static,
        blocking_mutex::raw::CriticalSectionRawMutex,
        Command,
    >,
) {
    let device_state_ticker = async {
        // make this more frequent in real life
        let mut ticker = Ticker::every(Duration::from_secs(1));

        const COMMANDS: [Command; 4] = [
            Command::DeviceState(DeviceStateCommand),
            Command::BatteryAndActiveTime(BatteryAndActiveTimeCommand),
            Command::DriveModeHistory(DriveModeHistoryCommand),
            Command::SystemStatus(SystemStatusCommand),
            // Command::UpdateProgress(UpdateProgressCommand),
        ];

        loop {
            ticker.next().await;

            for command in COMMANDS {
                let mut slot = cmd_sender.send().no_inline().await;
                *slot = command;
                slot.send_done();
            }
        }
    };

    embassy_futures::join::join_array([device_state_ticker])
        .no_inline()
        .await;
}
