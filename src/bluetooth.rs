// - USART2
// - rx: GPIOA 3 (bit 0x8) (`USART2_RX`)
// - tx: GPIOA 2 (bit 0x4) (`USART2_TX`)
// - 57600 baud

use at32f4xx_hal::{
    pac::USART2,
    uart::{Rx, Tx},
};
use deku::DekuContainerRead;
use embedded_io_async::{Read, Write};

// `[0x55, length, unknown, handler_idx_low, handler_idx_high, ...[data], crc[0], crc[1]]`

#[derive(deku::DekuRead, defmt::Format)]
#[deku(id_type = "u16", endian = "little")]
#[repr(u16)]
enum Endpoint {
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

#[derive(deku::DekuRead, defmt::Format)]
#[deku(magic = b"\0")]
enum Command {
}

#[embassy_executor::task]
pub async fn bluetooth_rx(rx: Rx<USART2, u8>) {
    bluetooth_rx_(rx).await;
}

async fn bluetooth_rx_(mut rx: Rx<USART2, u8>) {
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

        let (command, msg) = match Command::from_bytes((buf, 0)) {
            Ok(((rem, _), cmd)) => (cmd, rem),
            Err(_) => {
                defmt::warn!("Command parse error");
                continue;
            },
        };

        defmt::info!("Bluetooth RX: {} {}", command, buf);

        // if let Some(b) = <Rx<USART2, u8> as embedded_hal_nb::serial::Read>::read(&mut rx).ok() {
        //     defmt::info!("Bluetooth RX: {}", b);
        // } else {
        //     embassy_time::Timer::after_millis(100).await;
        // }
    }
}

#[embassy_executor::task]
pub async fn bluetooth_tx(tx: Tx<USART2, u8>) {
    bluetooth_tx_(tx).await;
}

async fn bluetooth_tx_(mut tx: Tx<USART2, u8>) {
    defmt::info!("Bluetooth TX startup");
    // loop {
    //     embassy_time::Timer::after_secs(1).await;
    // }
}
