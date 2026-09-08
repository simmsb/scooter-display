### Known opcodes

````java
private static final byte SET_LANGUAGE = 1;
private static final byte SET_DISPLAY_BRIGHTNESS = 2;
private static final byte SET_OPTION_UNLOCK_SCOOTER_DISPLAY = 3;
private static final byte SET_UNLOCK_CODE_DISPLAY = 4;
private static final byte SET_ACTIVATION = 5;
private static final byte SET_ACTIVATION_CODE = 6;
private static final byte SET_SPEED_LIMIT_ENABLED = 7;
private static final byte SET_SPEED_LIMIT = 8;
private static final byte SET_SPEED_UNIT = 9;
private static final byte SET_BLUETOOTH_ALWAYS_ON = 10;
private static final byte SET_SERIAL_NUMBER = 12;
private static final byte SET_AUTOMATIC_HEADLIGHTS = 13;
````

To set the PIN, send \[0x04, 0x30 + A, 0x30 + B, 0x30 + C, 0x30 + D\] to the settings write characteristic.
