## CAN IDs

|can id|extended|total|total distinct|sent by|
|------|--------|-----|--------------|-------|
|477||||update device (controlelr)|
|478||||update device (controller)|
|479||||controller|
|512||6030|16|controller|
|513||268542|615|controller|
|514||24152|92|controller|
|515||262806|21|controller|
|528||251240|1|controller|
|560||22793|8|controller|
|568||1|1|<possibly corruption>|
|768||349619|55|display|
|774||265621|275|display|
|775||11|1|<not in any firmware>|
|896||24|4|<not in any firmware>|
|900||6438|4948|update device (display)|
|903||24|2|<not in any firmware>|
|1024||265242|8|battery (assumed)|
|1025||261416|34616|battery (assumed)|
|1026||8760|279|battery|
|1027||8681|3|battery|
|1028||255852|33|battery|
|1029||8291|1|battery (assumed)|
|1536||28|1|<not in any firmware>|
|1537||15|7|<not in any firmware>|
|1539||1|1|<not in any firmware>|
|1542||27|1|<not in any firmware>|
|1543||1|1|<not in any firmware>|
|1792||895|2|display|
|1856|ext||||
|1857|ext||||
|1860|ext|53468|4|display|
|1861|ext|51818|7|battery|
|1862|ext|27079|1|display|
|1863|ext|865982|238|battery|
|1904||17541|4|display|
|1920||17266|4|controller|
|1984||2|2|<not in any firmware>|
|2040||2|2|\<not in any firmware.|

## Senders

### Sent by update devices

* 0x1dd ([477](CAN%20bus%20messages.md#477)) Firmware update (controller)
* 0x1de ([478](CAN%20bus%20messages.md#478)) Firmware update (controller)
* 0x384 ([900](CAN%20bus%20messages.md#900)) Firmware update (display)

### Sent by display unit

* 0x1ee ([494](CAN%20bus%20messages.md#494)) Trigger info dump
* 0x300 ([768](CAN%20bus%20messages.md#768)) Speed mode
* 0x306 ([774](CAN%20bus%20messages.md#774)) Throttle & Speed limit
* 0x700 ([1792](CAN%20bus%20messages.md#1792)) Firmware update progress note
* 0x740 ([1856](CAN%20bus%20messages.md#1856)) Unknown, not handled by controller, 2 bytes
* 0x742 ([1858](CAN%20bus%20messages.md#1858)) Unknown, not handled by controller
* 0x744 ([1860](CAN%20bus%20messages.md#1860)) Requests battery serial
* 0x746 ([1862](CAN%20bus%20messages.md#1862)) Request charge history
* 0x770 ([1904](CAN%20bus%20messages.md#1904)) Diagnostics/fwupd/findmy
* 0x771 ([1905](CAN%20bus%20messages.md#1905)) Diagnostics/fwupd/findmy
* 0x740 ([1856](CAN%20bus%20messages.md#1856)) Request for datetime

### Sent by controller

* 0x200 ([512](CAN%20bus%20messages.md#512)) Battery level, failure code
* 0x201 ([513](CAN%20bus%20messages.md#513)) Current speed
* 0x202 ([514](CAN%20bus%20messages.md#514)) Controller temperature, possibly motor current
* 0x203 ([515](CAN%20bus%20messages.md#515)) Controller status
* 0x210 ([528](CAN%20bus%20messages.md#528)) Is speed limit is 'enabled'
* 0x230 ([560](CAN%20bus%20messages.md#560)) Find-by related

### Sent by battery

* 0x400 ([1024](CAN%20bus%20messages.md#1024)) Command, state, estimated range
* 0x401 ([1025](CAN%20bus%20messages.md#1025)) Battery voltage and current
* 0x402 ([1026](CAN%20bus%20messages.md#1026)) Charge level
* 0x403 ([1027](CAN%20bus%20messages.md#1027)) State of health
* 0x404 ([1028](CAN%20bus%20messages.md#1028)) Battery capacity & temperature
* 0x405 ([1029](CAN%20bus%20messages.md#1029)) Firmware version
* 0x745 ([1861](CAN%20bus%20messages.md#1861)) Serial
* 0x747 ([1863](CAN%20bus%20messages.md#1863)) Charge history
* 0x741 ([1857](CAN%20bus%20messages.md#1857)) Current datetime or

## Usages

### 494

Triggers a dump of controller data on ID 495

Seems to always send \[0x60, 0, 0, 0, 0, 0, 0, 0\]

Controller returns with:

* Model number (LD.CR.S8)
* Hardware version (07.C.2.1)
* Firmware version (EG.2.2.1)
* Firmware build info (1)
* Some number of frames with just zeroes

### 478

Sets some flag if given a message of \[0xff, a, 0x13, b, c, d, e, f\] where some crc function over the entirebuffer = f

Used to enter firmware update mode

### 512

Doesn't have many values

Period of 3 seconds

|message|max|
|-------|---|
|0000000000000200|2025-10-16 13:26:14.790953|
|6200000000000100|2025-10-16 15:24:23.416665|
|6200000000000000|2025-10-16 15:24:29.42383|
|6200000000000200|2025-10-16 15:24:53.446186|
|7300000000000100|2025-10-17 12:25:23.906192|
|6400000000000000|2025-10-21 15:32:01.643574|
|6400000000000100|2025-10-21 15:32:04.650625|
|6400000000000200|2025-10-21 15:32:07.659688|
|0000000000000000|2025-10-21 15:58:54.275725|
|6300000000000000|2025-10-21 16:20:31.573279|
|6300000000000100|2025-10-21 17:38:03.213718|
|6300000000000800|2025-10-21 17:38:09.220196|
|6300000000000200|2025-10-21 17:38:15.225259|
|3f00000000000000|2025-11-04 17:56:42.214484|
|3f00000000000100|2025-11-04 17:56:45.217193|
|3f00000000000200|2025-11-04 17:56:48.232571|

Nibble 13 seems to count up to 3 slowly.
First byte changes sometimes

### 513

Contains motor speed and some status bits

`[motor_speed_low, motor_speed_high, 0, 0, status_bit]`

````
Scooter still and in eco+: 00 00 00 00 00
Scooter in walk mode:      0000000001
Scooter max speed of walk: 2002000001 (0x0220 -> 544, 6kmh)
Scooter max speed in eco+: a008000001 (0x08a0 -> 2208, 22kmh)  (2208 * 6/22 -> 602)

Last status bit:

01234567
.....x.. : Brake light on
......x. : Headlight on
.......x : Walk mode
````

### 514

Not read by display. Sent by controller.

Period of one second

Bytes:

````
   0: always 0
   1: Device temperature - 40
2..4: Either motor voltage, motor current (possibly read as voltage), 
4..8: always 0
````

### 515

Sets or reflects speed mode

Some bits of byte 6 change over time

Display reads first two bytes and stores them somewhere divided by 30. Never read.

````c
        state_ptr = (DAT_200002a0 & 0x3fff) << 2;
        can_tx_buffer[0] = (byte)state_ptr;
        can_tx_buffer[1] = (byte)((uint)state_ptr >> 8);
        can_tx_buffer[2] = (byte)DAT_2000029e;
        can_tx_buffer[3] = (byte)((ushort)DAT_2000029e >> 8);
        can_tx_buffer[4] = 0;
        can_tx_buffer[5] = can_tx_buffer[3] >> 6;
        if ((IS_IN_WALK_MODE == '\x06') || (DAT_200003c4 != '\0')) {
          sensor_value_uint = 0x1fff;
        }
        else if (LATEST_SPEED_MDOE == 0) {
          sensor_value_uint = 0x1fff;
        }
        else {
          sensor_value_uint = compute_speed_limiter((uint)BATTERY_PCT);
          sensor_value_uint &= 0xffff;
        }
        can_tx_buffer[6] = (byte)sensor_value_uint;
        can_tx_buffer[7] = (byte)((sensor_value_uint << 0x13) >> 0x1b);
        queue_can_message_for_tx((int)can_tx_buffer,8,0x203);
````

````
Walk : 000000000000ff1f
Eco  : 0000000000005700
Trip : 0000000000004500
Sport: 0000000000002f00
````

### 528

Sets speed limit 'mode' flag, seems to inform the display whether it should send speed limit in kph (45 = 45kmph limit), or as a 0 = 25, 1 = 35, 2 = 45 enum.

### 580

Emtpy other than counter in second byte, counts \[00, 20, 40, 60, 80, a0, c0, e0\]

### 768

Message sent from the display unit to the controller.

Contains the speed mode, and headlight state

Controller has tables of values for up to 9 speed modes. Dump from GT controller
seems to duplicate the values for sport for speed mode >=3 though.

Bytes:

````
0..2: speed mode [00a5, 015a, 025a, 035a] [00 165, 01 90, 02 90, 03 90]
   2: 0x64 = headlight on
3..5: always 5a64
   5: always 00
   6: speed mode [00, 1e, 32, 46] [00, 30, 50, 70]. not read
   7: counts up to 0xf when in walk mode

Reset       : 005a00a564006400
Locked      : 005a005a64000000
Walk        : 00a5005a6400000.
Eco         : 015a005a64001e00
Trip        : 025a005a64003200
Sport       : 035a005a64004600
Headlight on: 035a645a64004600
````

sending       005a00a564006400 causes the controller to reset

### 774

Message sent from the display unit to the controller.

Contains the throttle position, indicator light bits, and speed limit.

Throttle = msg\[0\] + (msg\[1\] & 1) * 0x100

Throttle range: 0x1d .. 0x191  (29 .. 401)

With throttle off: 57000000 02000000 (0x0057 -> 87?)
With throttle max: 67010000 02000000 (0x0167 -> 359?)

Speed limit appears to be unused on GT controller firmware.

Bytes:

````
0   : Throttle position
1   : Bitset:
      01234567
      .......x : Throttle msb
      ......x. : Left blinker
      .....x.. : Right blinker
2   : Always zero
3   : Speed limit. 0 = 25, 1 = 35, 2 = 45
4   : Unknown, always 0x02
5..8: Unknown, always 0x00_00_00
````

### 900

Sending 'C' 5 times triggers some reset. Probably entering update mode.

Sending \[0xaa, 0x04, 0x04, 0x52, 0x45, 0x50, 0x07\] erases some flash regions and resets.

### 1024

0..2: 'command registers'
2..4: 'state registers'
4..8: 'estimated range'

````
At idle  : 0140084000000000
At walk  : 0140084000000000
(initial): 0140284000000000
At eco+  : 01002c4000000000
````

### 1025

Battery voltage and current.

0..4: voltage (mV)
4..8: current (mA)

### 1026

Battery relative and absolute state of charge.

0..4: Relative state of charge (%)
4..8: Absolute state of charge (mAh)

If following condition is satisfied we store the first byte somewhere:

````
         ((uint)CAN_400_MSG_BUF[3] << 0x18 | (uint)CAN_400_MSG_BUF[2] << 0x10 |
          (uint)CAN_400_MSG_BUF[1] << 8 | (uint)CAN_400_MSG_BUF[0]) < 0x65)
````

### 1027

Battery state of health

0..1: relative soh (%)
4..8: absolute soh (mAh)

### 1028

0..2: capacity (0x4e20 = 20,000)
2..4: unknown/unused
4: battery charged
5..6: unknown/unused
6..8: battery temp

(high byte in 7, low in 6)
Firmware stores on receipt (bytes 7&6 - 2731) / 10

1:  2741: \[181, 10\];
2:  2751: \[191, 10\]
50: \[159, 12\]
150: \[135, 16\]

|message|
|-------|
|204e00000100be0b|
|204e00000000b70b|
|204e00000000b60b|
|204e00000000b50b|
|204e00000000b40b|
|204e00000000b30b|
|204e00000000b20b|
|204e00000000b10b|
|204e00000000b00b|
|204e00000000af0b|
|204e00000000ae0b|
|204e00000000ad0b|
|204e00000000ac0b|
|204e00000000ab0b|
|204e00000000aa0b|
|204e00000000a90b|
|204e00000000a80b|
|204e00000000a70b|
|204e00000000a00b|
|204e000000009f0b|
|204e000000009e0b|
|204e000000009d0b|

### 1029

Battery firmware/ hardware version.

3.7.0

Always 1b0b000000000000

### 1792

Device sends this to note progress when firmware is being updated.

### 1858

Display sends this with a payload of \[0, 0\]

Not read by controller

Doesn't seem to be sent

### 1860

Is a request for a response of ID 1861

Counts to 4

### 1861

Has four known values

Might be some VIN, or the controller ID?

|message|max|
|-------|---|
|02153c3230303737|2025-10-17 12:25:23.426897|
|0315325d32365730|2025-10-17 12:25:23.933492|
|01155720505d5436|2025-10-17 12:25:24.933159|
|0215383230303537|2025-11-04 17:56:46.589356|
|0315325932345730|2025-11-04 17:56:47.091302|
|0415313031|2025-11-04 17:56:47.595324|
|0115572050595434|2025-11-04 17:56:48.090305|

0115572050595434 02153c3230303737 0315325d32365730
01155720505d5436 0215383230303537 0315325932345730
x   x      x       x          x   x

1001 0100 1100 0111 0100 0110
1101 0110 1000 0101 1001 0100

### 1862

Always 01fe

Is a request for a response of ID 1863

### 1863

Charge history from battery

After 1862, responds with multiples of two messages:

* \[idx, unknown, year - 2000, month, day, hour minute, second\]
* \[idx, unknown, charge_low, charge_high\]

### 1856

Is a request for a response from [1857](CAN%20bus%20messages.md#1857)

Messages:

* \[4b, 00\]
* \[4a, 00\]

### 1857

Is either current timestamp from the battery RTC, or some other number that ends up in the system status bluetooth response.

Examples:

* Timestamp: \[4b, 00, 00, 00, 69, 99, 52, 47\]

* Other unknown: \[4a, 00, 00, 00, 1d, 00\]

### 1904

Might be find my related?

Is a request for a response from 1920

Commands:

* 01XX: Cause controller to dump XX..XX+5 bytes from some region. Response is \[01, remainder | 0x10, XX, a, b, c, d, e\].
* 03XX: Same as 01XX, just a different region.
* 04XX: Same
* 05XX: Always returns \[05, 0x10, 0, YY or 0xff, 0...\]
* 06XX: Always returns \[06, 0x10, 0, EXPECTED_LENGTH != 0xff, some_flag, some_flag_2, some_value_3, 0\]

EXPECTED_LENGTH of 06XX is length of 04XX

### 1905

Firmware update/ findmy related

### 1920

Only two values: 0x0610000100000000, 0x0510006400000000
