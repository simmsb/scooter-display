use embedded_graphics::prelude::RgbColor;

pub type ColorFormat = embedded_graphics::pixelcolor::Rgb565;

pub const GREEN: ColorFormat = ColorFormat::new(20, 200, 50);
pub const RED: ColorFormat = ColorFormat::new(255, 0, 0);
pub const YELLOW: ColorFormat = ColorFormat::new(255, 255, 0);
pub const BLUE: ColorFormat = ColorFormat::new(100, 210, 255);
pub const BLACK: ColorFormat = ColorFormat::new(0, 0, 0);
pub const GREY: ColorFormat = ColorFormat::new(150, 150, 150);
pub const WHITE: ColorFormat = ColorFormat::WHITE;

pub const BACKGROUND: ColorFormat = WHITE;
pub const SECONDARY_BACKGROUND: ColorFormat = ColorFormat::new(200, 200, 200);
pub const CONTENT: ColorFormat = BLACK;
pub const SECONDARY_CONTENT: ColorFormat = ColorFormat::new(50, 50, 50);
