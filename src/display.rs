use core::convert::Infallible;

use at32f4xx_hal::timer::PwmChannel;
use at32f4xx_hal::{
    gpio::{Output, Pin},
    timer::SysDelay,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::{Dimensions, Point, Size, Transform as _},
    primitives::Rectangle,
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

    #[inline(always)]
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
pub type InnerDisplay = mipidsi::Display<
    mipidsi::interface::ParallelInterface<BusAsU16<'B', 0, 0xFFFF>, DcPin, WrPin>,
    mipidsi::models::ST7796,
    RstPin,
>;
pub type Color = <InnerDisplay as DrawTarget>::Color;

// 0,0     : top right
// 320,0   : top left
// 320,480 : bottom left
pub struct Display {
    // CS, needs to be low for the display to accept commands?
    _cs_pin: CsPin,
    // RD (data read), unused for now, should stay high
    _rd_pin: RdPin,
    pub inner: InnerDisplay,
    backlight: Backlight,
    partial_buf: &'static mut [Color],
}

impl Dimensions for Display {
    fn bounding_box(&self) -> Rectangle {
        self.inner.bounding_box()
    }
}

impl DrawTarget for Display {
    type Color = <InnerDisplay as DrawTarget>::Color;

    type Error = <InnerDisplay as DrawTarget>::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.inner.draw_iter(pixels)
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        if area.size.width == 0 {
            return Ok(());
        }

        let rows_per_chunk = self.partial_buf.len() / (area.size.width as usize);
        let slice = &mut self.partial_buf[..(rows_per_chunk * area.size.width as usize)];

        // partial fb too small, fallback to non buffered
        if rows_per_chunk == 0 {
            return self.inner.fill_contiguous(area, colors);
        } else {
            let mut colors = colors.into_iter();
            let chunks = area.size.height / rows_per_chunk as u32;
            let remainder = area.size.height % rows_per_chunk as u32;

            let chunked_rect = Rectangle::new(
                area.top_left,
                Size::new(area.size.width, rows_per_chunk as u32),
            );

            for n in 0..chunks {
                for p in slice.iter_mut() {
                    let Some(c) = colors.next() else {
                        return Ok(());
                    };
                    *p = c;
                }

                self.inner.fill_contiguous(
                    &chunked_rect.translate(Point::new(0, n as i32)),
                    slice.iter().copied(),
                )?;
            }

            let remainder_rect = Rectangle::new(
                area.top_left + Point::new(0, chunks as i32 * rows_per_chunk as i32),
                Size::new(area.size.width, remainder),
            );

            self.inner.fill_contiguous(&remainder_rect, colors)
        }
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.inner.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.inner.clear(color)
    }
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
    partial_buf: &'static mut [Color],
) -> Display {
    cs.set_low();
    rd.set_high();

    let bus1 = BusAsU16 { inner: bus };
    let interface = mipidsi::interface::ParallelInterface::new(bus1, dc, wr);
    let mut display = mipidsi::Builder::new(mipidsi::models::ST7796, interface)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .orientation(mipidsi::options::Orientation {
            // Deg0 for actual use, 180 on my desk
            rotation: mipidsi::options::Rotation::Deg180,
            mirrored: true,
        })
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .init(delay)
        .unwrap();

    display.set_tearing_effect(mipidsi::options::TearingEffect::Vertical);

    Display {
        _cs_pin: cs,
        _rd_pin: rd,
        inner: display,
        backlight,
        partial_buf,
    }
}
