## Bluetooth [UART protocol](../../System%20communication/Bluetooth/UART%20protocol.md)

* USART2
* rx: GPIOA 3 (bit 0x8) (`USART2_RX`)
* tx: GPIOA 2 (bit 0x4) (`USART2_TX`)
* 57600 baud

## NFC

* USART1
* rx: GPIOA 10 (bit 0x400) `IN_PU` (`USART1_RX`)
* tx: GPIOA 9 (bit 0x200) `AF_PP` (`USART1_TX`)
* 115200 baud

## Buttons (control unit)

* UART5
* rx: GPIOD 2 (0x4) (`UART5_RX`)
* tx: GPIOC 12 (0x1000) (`UART5_TX`)
* 9600 baud

### Messages

#### Mode 1

* `BUTTON_DATA[2] = BUTTON_UART_BUF[0]`

#### Mode 4

* if `BUTTON_UART_BUF[0] ^ BUTTON_UART_BUF[2] == 0xff` and `BUTTON_UART_BUF[1] ^ BUTTON_UART_BUF[3] == 0xff`
* `BUTTON_DATA[0] = BUTTON_UART_BUF[0]`
* `BUTTON_DATA[1] = BUTTON_UART_BUF[1]`

````
  POWER_PRESSED = (BUTTON_DATA[0] & 1) == 0;
  UP_PRESSED = (BUTTON_DATA[0] & 2) == 0;
  DOWN_PRESSED = (BUTTON_DATA[0] & 4) == 0;
  L_PRESSED = (BUTTON_DATA[0] & 8) == 0;
  R_PRESSED = (BUTTON_DATA[1] & 1) == 0;
````

## SPI

* GPIOA pins 4, 5, 6, and 7

GPIO 4 (bit 0x10): `OUT_PP` (bit 0x10): CS (`SPI1_CS`)
GPIO 5 (bit 0x20) & GPIO 7 (bit 0x80): `AF_PP` (bits 0xa0) (5: `SPI1_SCK`, 7: `SPI1_MOSI`)
Pin 6: `IN_Floating` (bit 0x40) (`SPI1_MISO`)

````
  spi_config.SPI_TransMode = 0;
  spi_config.SPI_Mode = 0x104;
  spi_config.SPI_FrameSize = 0;
  spi_config.SPI_CPOL = 2;
  spi_config.SPI_CPHA = 1;
  spi_config.SPI_NSSSEL = 0x200;
  spi_config.SPI_MCLKP = 0;
  spi_config.SPI_FirstBit = 0;
  spi_config.SPI_CPOLY = 7;
````

SPI uses DMA

## CAN [CAN Bus overview](../../System%20communication/CAN/README.md)

rx: GPIOA 11 (bit 0x800) (`CAN1_RX`)
tx: GPIOA 12 (bit 0x1000) (`CAN1_TX`)

250k

````
  can_config.CAN_TTC = DISABLE; // time triggered communication
  can_config.CAN_ABO = ENABLE; // automatic bus off
  can_config.CAN_AWU = ENABLE; // automatic wake up
  can_config.CAN_NART = DISABLE; // no automatic retransmission
  can_config.CAN_RFL = ENABLE; // receive fifo locked
  can_config.CAN_TFP = ENABLE; // transmit fifo clocked
  can_config.CAN_Mode = '\0';  // 0 = init, 1 = normal, 2 = sleep
  can_config.CAN_SJW = '\0';  // 1 time quantumn
  can_config.CAN_BS1 = '\a'; // 11 time quantumn
  can_config.CAN_BS2 = '\x02'; // 3 time quantumn
  can_config.CAN_Prescaler = 0x18; // length of time quantumn
````

## Some other GPIOs configured

IOMUX bit 25 (mask 0x2000000) is set, disables jtag (swd remains), allows usage of GPIOA 15, GPIOB 3, GPIOB 4
IOMUX bit 8 (mask 0x100) is set, CH1/EXT/PA15, CH2/PB3, CH3/PA2 and CH4/PA3

GPIOC 4 (bit 0x10), input pullup. Can be `ADC1_IN14` or `SDIO1_CK` but probably doesn't use those.

GPIOF 4 (bit 0x10), out pp: Probably NFC reset

GPIOA 0 (bit 0x01), input pullup (`WKUP`)

GPIOA 1 (bit 0x02), input pullup: power button. This is also wired up to the controller and is used to start it.

## Display pins.

GPIOA 8 (bit 0x100), out pp. Might be backlight enable
GPIOA 15 (bit 0x8000), out pp: backlight pwm, IOMUX alt mapped `TMR2_CH1`.

GPIOB (all pins), output

GPIOC 0 (bit 0x1), output, starts high, Read clock
GPIOC 1 (bit 0x2), output, starts high, display reset.

GPIOC 13 (bit 0x2000): display command, starts high - goes low when sending anything
GPIOC 14 (bit 0x4000): display command, starts high, probably dc
GPIOC 15 (bit 0x8000): display command, starts high, probably wr

TODO: locate display pins:

1. DC: goes low when sending a command over the parallel bus, high when sending data
1. WR: goes low when changing bus values, high when done
1. Backlight, could be backlight enable + pwm, could be just one

### Display striping sequence

````
GPIOC.CLR = 0x2000
GPIOC.SCR = 1
GPIOC.SCR = 0x4000;
GPIOC.CLR = 0x8000;
GPIOB.ODT = data;
GPIOC.SCR = 0x8000;
GPIOC.SCR = 0x2000;
````

## ADC

````
  local_18.ADC_Mode = 0;
  local_18.ADC_ScanMode = DISABLE;
  local_18.ADC_ContinuousMode = DISABLE;
  local_18.ADC_ExternalTrig = 0xe0000;
  local_18.ADC_DataAlign = 0;
  local_18.ADC_NumOfChannel = '\x03';
````

### Channels

GPIOC 2 (bit 0x04), analog input. `ADC1_IN12`
GPIOC 3 (bit 0x08), analog input. `ADC1_IN13`
GPIOC 5 (bit 0x20), analog input. `ADC1_IN15`

### 12 (0xc) (Index 0): Ambient light sensor

Peak (torch shining): 4096
Min (sensor obscured): 900

### 13 (0xd) (Index 1): Throttle voltage

Peak (throttle fully depressed): 2820
Min (throttle untouched): 730

### 15 (0xf) (Index 2): Battery voltage, reads ~238 on my bench

### Scope probing

PF7 - GND
PF6 - High - UNKNOWN!!!
PA13 - swdio
PA12 - Clock - CAN
PA11 - Clock - CAN
PA10 - High - NFC Uart
PA9 - High - NFC Uart
PA8 - Low
PC9 - Low
PC8 - Low
PC7 - Low
PC6 - Low

PA7 - High - UNKNOWN!!!
PA4 - High - UNKNOWN!!!
PF4 - Low - NFC Reset
PA1 - High - Button, input pullyp
PA2 - Data - Bluetooth uart tx
PA0 - High - known input pullup
