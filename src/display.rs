use core::convert::Infallible;

use at32f4xx_hal::pac::GPIOB;
use at32f4xx_hal::{
    gpio::{Output, Pin},
    pac::GPIOC,
    timer::SysDelay,
};
use mipidsi::interface::InterfaceKind;

pub struct BusAsU8<const P: char, const SHIFT: u8, const MASK: u16> {
    inner: at32f4xx_hal::gpio::Bus<P, SHIFT, MASK, Output>,
}

pub struct BusAsU16<const P: char, const SHIFT: u8, const MASK: u16> {
    // cs: CsPin,
    inner: at32f4xx_hal::gpio::Bus<P, SHIFT, MASK, Output>,
}

impl<const P: char, const SHIFT: u8, const MASK: u16> mipidsi::interface::OutputBus
    for BusAsU8<P, SHIFT, MASK>
{
    type Word = u8;

    const KIND: mipidsi::interface::InterfaceKind = InterfaceKind::Parallel8Bit;

    type Error = Infallible;

    fn set_value(&mut self, value: Self::Word) -> Result<(), Self::Error> {
        self.inner.set_state(value as u16);
        Ok(())
    }
}

impl<const P: char, const SHIFT: u8, const MASK: u16> mipidsi::interface::OutputBus
    for BusAsU16<P, SHIFT, MASK>
{
    type Word = u16;

    const KIND: mipidsi::interface::InterfaceKind = InterfaceKind::Parallel16Bit;

    type Error = Infallible;

    fn set_value(&mut self, value: Self::Word) -> Result<(), Self::Error> {
        // defmt::info!("Striping: {}", value);
        self.inner.set_state(value);
        Ok(())
    }
}

// GPIOC 0 (bit 0x1), output, starts high, Read clock
// GPIOC 1 (bit 0x2), output, starts high, display reset.

// GPIOC 13 (bit 0x2000): display command, starts high - goes low when sending anything
// GPIOC 14 (bit 0x4000): display command, starts high, probably dc
// GPIOC 15 (bit 0x8000): display command, starts high, probably wr

type Bus = at32f4xx_hal::gpio::Bus<'B', 0, 0xFFFF, Output>;
// type CsPin = Pin<'C', 13, Output>;
type DcPin = Pin<'C', 14, Output>;
// type RdPin = Pin<'C', 0, Output>;
type WrPin = Pin<'C', 15, Output>;
type RstPin = Pin<'C', 1, Output>;

type Display = mipidsi::Display<
    mipidsi::interface::ParallelInterface<BusAsU16<'B', 0, 0xFFFF>, DcPin, WrPin>,
    mipidsi::models::ST7796,
    RstPin,
>;

pub fn init(
    // cs: CsPin,
    dc: DcPin,
    // rd: RdPin,
    wr: WrPin,
    rst: RstPin,
    bus: Bus,
    delay: &mut SysDelay,
) -> Display {
    let bus1 = BusAsU16 { inner: bus };
    let interface = mipidsi::interface::ParallelInterface::new(bus1, dc, wr);
    mipidsi::Builder::new(mipidsi::models::ST7796, interface)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .orientation(mipidsi::options::Orientation { rotation: mipidsi::options::Rotation::Deg90, mirrored: false })
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .init(delay)
        .unwrap()
}
