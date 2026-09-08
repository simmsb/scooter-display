# Egret GT bluetooth protocol

The bluetooth chip speaks to the display uC over UART.

## Bluetooth -> display uC

The BT chip sends two message types.

* `[0x43]`: Used to enter firmware update mode, and perform updates. The firmware update data is the exact same as CAN.
* `[0x55, ...]`: Used to inform the display of writes to some attribute.

For `0x55` a message is:

`[0x55, length, unknown, handler_idx_low, handler_idx_high, ...[data], crc[0], crc[1]]`

Length is the total length of the packet, CRC and `0x55` included.
The crc is calculated in the same way as updates over the can bus: `CRC_16_XMODEM(buf[0..len-2])`.

## display uC -> Bluetooth

Same form as rx, but first byte is `0xaa`.

`[0xaa, length, 0, handler_idx_low, handler_idx_high, ...[data], crc[0], crc[1]]`

## Sequencing

The uC always acks any message sent to it by the display with a zero length
message with the same handler idx as the sent message.

````
0 <display -> uc>: 9      01000000000000000000640001FEFF                   ..........d.... 3589 (3589)
4 <uc -> display>: 9                                                        56784 (56784)
````

However the display does not ack messages sent to it by the uc.

For READ attributes, the uc sends a zero byte message to initiate a read.

## Handler IDs

0. Unknown, has no callbacks in firmware
0. Unknown, has no callbacks in firmware
0. Returns bit 29 of some bit flags stored on the display. Seems to be the 'bluetooth enabled' bit
0. Used to set connection state
0. Unknown, tx callback is noo
0. Device information (VIN, device name, firmware versions
0. System status (battery level, power output, etc
0. System status (unknown fields
0. 'operation' command handle
0. Device state (power/locked/lights/charging/range
0. Odometer
0. 'settings' command handler
0. Current 'settings' report
0. Set customer name
0. Report customer name
0. Unknown. Seems to do nothing
0. Might be current speed
0. Reports charge history
0. Reports failure code
0. Reports battery percent and total time active
0. Reports time spent in each drive mode + battery
0. Unknown, does nothing
0. Unknown, does nothing
0. Might be update progress
   1. Possibly also used to initiate pairing process [Todolist > ^53d77f](../../Todolist.md#53d77f)
   1. Buffer is 8 bytes long, pairing mode might be sending \[1, 0, 0, 0, 0, 0, 0, 0\] or \[1, 1, 0, 0, 0, 0, 0, 0\]
0. Paired or other connected status
0. Initiates bluetooth update
0. Reports bluetooth update state
