# Summary

This file documents the firmware update process for the display unit.

The process for other components is likely similar, likely just with differing CAN IDs.

# Updates

The device has a bootloader up to 0x8000. The main program is loaded from the
flash chip to 0x8000 and then the bootloader jumps to it.

Firmware update mode is initiated by sending CAN 0x384 (900) with the single
byte contents \[0x43\] until the device responds on CAN 0x700 (1792) with a single byte \[0x6\]

On the device, sending this 0x43 byte triggers a system reset. It seems the
bootloader picks up on further 0x43 messages and enters its flashing mode.

The firmware update is transferred in chunks.

The updater sends a total of 69 bytes over 9 can frames (always ID 0x384). Each
chunk starts with one frame that contains a literal `1`, followed by the
sequence number, the inverse of the sequence number, and then the first 5 bytes
of data.

This sequence number is incremented for each chunk, wrapping around back to 0 at
0xff.

The last two bytes of the last frame of the chunk are a CRC.

Each chunk contains 64 bytes of firmware data.

The CRC is CRC-16-CCITT/CCITT-ZERO (that is, crc-16 with a polynomial of 0x1021
and all other parameters empty. Might also be known as `CRC_16_XMODEM`) over the
bytes of the payload.

frame 1: \[1, seq, 0xff - seq, data\[0\], data\[1\], data\[2\], data\[3\], data\[4\]\]
frame 2: data\[5  .. 13\]
frame 3: data\[13 .. 21\]
frame 4: data\[21 .. 29\]
frame 5: data\[29 .. 37\]
frame 6: data\[37 .. 45\]
frame 7: data\[45 .. 53\]
frame 8: data\[53 .. 61\]
frame 9: \[data\[61\], data\[62\], data\[63\], crc\[0\], crc\[1\]\]

The first chunk is the update parameters. That is, the filename (This might need
to be `AT_R2_JHZY_GT1_GE_FM_HW02_4.0.2`) as a null terminated string, followed
by the firmware length as a null terminated string.

An example first chunk is:

````
updater->device 900:  [1, 0, ff, 41, 54, 5f, 52, 32]
updater->device 900:  [5f, 4a, 48, 5a, 59, 5f, 47, 54]
updater->device 900:  [31, 5f, 47, 45, 5f, 46, 4d, 5f]
updater->device 900:  [48, 57, 30, 32, 5f, 34, 2e, 30]
updater->device 900:  [2e, 32, 0, 31, 33, 37, 37, 38]
updater->device 900:  [30, 0, 0, 0, 0, 0, 0, 0]
updater->device 900:  [0, 0, 0, 0, 0, 0, 0, 0]
updater->device 900:  [0, 0, 0, 0, 0, 0, 0, 0]
updater->device 900:  [0, 0, 0, 93, cf]
device->updater 1792: [6]
device->updater 1792: [6]
device->updater 1792: [6]
device->updater 1792: [6]
````

For some reason, the device responds with four CAN 0x700 (1792) messages
containing \[6\] after the first chunk. For subsequent chunks, it only responds
with one \[6\] message per chunk. If the device responds with a 0x700 message of
anything else, the crc failed.

After this, the full application firmware is streamed.

The last chunk needs to be padded with zeroes to fit the 69 byte length as the
checksum is always read from bytes 68 and 69.

The app firmware is a full application firmware with the interrupt table, etc.

After transmitting the firmware, a final 0x384 message is sent with the payload
`[4]`, which will trigger a reboot.
