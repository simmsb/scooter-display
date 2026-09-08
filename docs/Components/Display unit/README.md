* Shows status of scooter on display.
* Handles input from following components:
  * Button pad
  * Throttle
  * External light sensor
  * NFC reader
  * Bluetooth

It participates on the can bus, and sends these messages: [CAN bus messages > Sent by display unit](../../System%20communication/CAN/CAN%20bus%20messages.md#sent-by-display-unit)

The hardware used on this is documented in [Display unit hardware](Display%20unit%20hardware.md), and the pinout in [Display unit MCU pinouts](Display%20unit%20MCU%20pinouts.md)

## General operation

The default firmware seems to perform the following tasks:

* Reads throttle with an ADC and transmits the throttle position to the scooter [CAN bus messages > 774](../../System%20communication/CAN/CAN%20bus%20messages.md#774)
* Informs display of current speed mode [CAN bus messages > 768](../../System%20communication/CAN/CAN%20bus%20messages.md#768)

## Firmware

The default firmware consists of a bootloader (0..0x8000) and a main application (0x8000..).
The bootloader handles the update process ([Firmware updates over CAN](../../System%20communication/CAN/Firmware%20updates%20over%20CAN.md)), the main application reboots when it notices CAN messages initiating an update.

### Details of default firmware

It uses segger EMWin for the UI. A global variable is used to determine what the current page is (main dashboard, settings, subsetting page, etc). There are a couple of debug pages that appear unreachable in this firmware \[^1\](no code path writes to the global with the)\[^2\]

Values such as the speed, battery level and light level are averaged by recording results in a ring buffer.

It has a table of CAN handlers, and a table of bluetooth handlers.

Images used in the default firmware are stored on a flash chip, when the device wants to draw an image to the display, it streams the image data from the flash to the display.
