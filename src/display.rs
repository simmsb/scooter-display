use core::convert::Infallible;

use at32f4xx_hal::{
    gpio::OutputPin,
    pac::GPIOB,
    timer::{PwmChannel, Timer2},
};
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
        self.inner.set_state(value);
        Ok(())
    }
}

pub type Bus = at32f4xx_hal::gpio::Bus<'B', 0, 0xFFFF, Output>;
pub type CsPin = Pin<'C', 13, Output>;
pub type DcPin = Pin<'C', 14, Output>;
pub type RdPin = Pin<'C', 0, Output>;
pub type WrPin = Pin<'C', 15, Output>;
pub type RstPin = Pin<'C', 1, Output>;
pub type Backlight = PwmChannel<at32f4xx_hal::pac::TMR2, 0>;

// 0,0     : top right
// 320,0   : top left
// 320,480 : bottom left
pub struct Display {
    // CS, needs to be low for the display to accept commands?
    cs_pin: CsPin,
    // RD (data read), unused for now, should stay high
    rd_pin: RdPin,
    pub inner: mipidsi::Display<
        mipidsi::interface::ParallelInterface<BusAsU16<'B', 0, 0xFFFF>, DcPin, WrPin>,
        mipidsi::models::ST7796,
        RstPin,
    >,
    backlight: Backlight,
}

impl Display {
    pub fn backlight_level(&mut self, level: u8) {
        let duty = self.backlight.get_max_duty().saturating_div(level as u16);
        self.backlight.set_duty(duty);
    }
}

pub fn init(
    mut rd: RdPin,
    mut cs: CsPin,
    dc: DcPin,
    wr: WrPin,
    rst: RstPin,
    bus: Bus,
    delay: &mut SysDelay,
    backlight: Backlight,
) -> Display {
    cs.set_low();
    rd.set_high();

    let bus1 = BusAsU16 { inner: bus };
    let interface = mipidsi::interface::ParallelInterface::new(bus1, dc, wr);
    let display = mipidsi::Builder::new(mipidsi::models::ST7796, interface)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .orientation(mipidsi::options::Orientation {
            rotation: mipidsi::options::Rotation::Deg90,
            mirrored: false,
        })
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .init(delay)
        .unwrap();

    Display {
        cs_pin: cs,
        rd_pin: rd,
        inner: display,
        backlight,
    }
}
