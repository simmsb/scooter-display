## MCU

AT32F415 (stm32f101x clone)

Datasheet: [AT32F415 Datasheet.pdf](../../Documentation/Datasheets/AT32F415%20Datasheet.pdf)
Register map: [AT32F415 Reference.pdf](../../Documentation/Datasheets/AT32F415%20Reference.pdf)
Corresponding stm32 reference: [STM32F101x Reference.pdf](../../Documentation/Datasheets/STM32F101x%20Reference.pdf)

The peripheral registers are roughly 1-1 with the stm32f1 registers, but all names are different. With both register map datasheets open, it is possible to compare the registers as all memory locations are the same.

The RTC peripheral appears to be the same as that found in the stm32f3.

## SPI Flash

Datasheet: [W25Q128FV.pdf](../../Documentation/Datasheets/W25Q128FV.pdf)

There's a SPI flash chip (GigaDevices w25q128) connected to the display.
It contains the bitmaps for all text and images used on the display in the default firmware.

The default firmware also seems to write to it as part of some update process (perhaps part of bluetooth?). But I haven't yet discovered how the write protect pin of the flash is de-asserted - it seems to be held high externally.

## Bluetooth

CH573 running some 'bluetooth proxy' firmware. See [UART protocol](../../System%20communication/Bluetooth/UART%20protocol.md)

## Display

Some st7796

Data is transferred in parallel over all GPIOB pins. (See [Display unit MCU pinouts > Display pins.](Display%20unit%20MCU%20pinouts.md#display-pins))
