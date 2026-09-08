The motor controller handles all motor related functions (speed limiting, acceleration profiles)

The display unit ([Display unit overview](../Display%20unit/README.md)) informs the motor controller of the throttle position, speed mode (gear / drive mode), and speed limit (only on GTS).

The motor controller reports:

* Battery level
* Current speed
* Motor current/ battery voltage
* Battery level (however the battery also transmits the level on the bus, so this is redundant)

## Firmware

Like the [Display unit overview](../Display%20unit/README.md), the controller has a bootloader as 0x8000000.

It also has a fallback application at 0x8003000 and the main application at 0x8006200

At 0x8006000 is a 0x200 byte string containing the application code length as a base-10 string. The string is padded with spaces (0x20) until the end of the 512 byte buffer, which ends in \r\n.
