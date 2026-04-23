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
use embedded_io_async::Write as _;
use static_cell::StaticCell;

// `[0x55, length, unknown, handler_idx_low, handler_idx_high, ...[data], crc[0], crc[1]]`

static COMMAND_CHANNEL: StaticCell<
    zerocopy_channel::Channel<'static, blocking_mutex::raw::ThreadModeRawMutex, Command>,
> = StaticCell::new();
static COMMAND_BUF: StaticCell<[Command; 4]> = StaticCell::new();

pub fn start_bluetooth(spawner: Spawner, uart: Serial2) {
    let (bt_tx, bt_rx) = uart.split();
    let buf = COMMAND_BUF.init([const { Command::Unknown0(Unknown0Command) }; 4]);
    let (cmd_tx, cmd_rx) = COMMAND_CHANNEL
        .init(zerocopy_channel::Channel::new(buf))
        .split();
    spawner.spawn(bluetooth_rx(bt_rx, cmd_tx).unwrap());
    spawner.spawn(bluetooth_tx(bt_tx, cmd_rx).unwrap());
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
            Ok(((rem, _), cmd)) => cmd,
            Err(e) => {
                defmt::warn!(
                    "Command parse error: {} (in: {})",
                    defmt::Debug2Format(&e),
                    buf
                );
                continue;
            }
        };

        defmt::info!("Bluetooth RX: {} {}", command, buf);

        *cmd_sender.send().await = command;
        cmd_sender.send_done();

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
) {
    bluetooth_tx_(tx, cmd_receiver).await;
}

async fn bluetooth_tx_(
    mut tx: Tx<USART2, u8>,

    mut cmd_receiver: zerocopy_channel::Receiver<
        'static,
        blocking_mutex::raw::ThreadModeRawMutex,
        Command,
    >,
) {
    defmt::info!("Bluetooth TX startup");

    // TODO: calc max size
    let mut buf = [0u8; 100];

    loop {
        let cmd = cmd_receiver.receive().await;

        let resp = match cmd {
            Command::Unknown0(_) => None,
            Command::Unknown1(_) => None,
            Command::BluetoothEnabledBit(_) => None,
            Command::SetConnectionState(_) => None,
            Command::Unknown4(_) => None,
            Command::DeviceInformation(_) => {
                Some(Response::DeviceInformation(DeviceInformationResponse))
            }
            Command::SystemStatus(_) => None,
            Command::SystemStatusUnknown(_) => {
                Some(Response::SystemStatusUnknown(SystemStatusUnknownResponse))
            }
            Command::OperationHandle(_) => None,
            Command::DeviceState(_) => None,
            Command::Odometer(_) => None,
            Command::SettingsHandler(_) => None,
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
            Command::BatteryAndActiveTime(_) => None,
            Command::DriveModeHistory(_) => None,
            Command::Unknown21(_) => None,
            Command::Unknown22(_) => None,
            Command::UpdateProgress(_) => None,
            Command::ConnectedStatus(_) => None,
            Command::InitiateBluetoothUpdate(_) => None,
        };

        cmd_receiver.receive_done();

        if let Some(resp) = resp {
            let to_send = match crate::framed_reader::assemble_framed_deku(&mut buf, 0xaa, &resp) {
                Err(e) => {
                    defmt::error!("Error writing bt response frame: {}", e);

                    continue;
                }
                Ok(to_send) => to_send,
            };

            let _ = tx.write_all(to_send).await;
        }
    }
}
