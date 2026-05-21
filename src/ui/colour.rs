use embedded_graphics::prelude::RgbColor;

pub type ColorFormat = embedded_graphics::pixelcolor::Rgb565;

// pub const GREEN: ColorFormat = ColorFormat::new(20, 200, 50);
// pub const RED: ColorFormat = ColorFormat::new(255, 0, 0);
// pub const YELLOW: ColorFormat = ColorFormat::new(255, 255, 0);
// pub const BLUE: ColorFormat = ColorFormat::new(100, 210, 255);
pub const BLACK: ColorFormat = ColorFormat::new(0, 0, 0);
// pub const GREY: ColorFormat = ColorFormat::new(150, 150, 150);
// pub const WHITE: ColorFormat = ColorFormat::WHITE;

/// Rgb565::new expects inputs to be in the range of the channel, not 8 bits.
/// So we need to scale them.
const fn rgb(r: u8, g: u8, b: u8) -> ColorFormat {
    let r = (r as u16 * ColorFormat::MAX_R as u16) / u8::MAX as u16;
    let g = (g as u16 * ColorFormat::MAX_G as u16) / u8::MAX as u16;
    let b = (b as u16 * ColorFormat::MAX_B as u16) / u8::MAX as u16;

    ColorFormat::new(r as u8, g as u8, b as u8)
}

macro_rules! colour {
    ($name:ident: $r:expr, $g:expr, $b:expr) => {
        #[doc = concat!(
                    "<div style=\"margin:2px 0\">",
                    "<span style=\"background-color:rgb(",
                    stringify!($r),
                    ",",
                    stringify!($g),
                    ",",
                    stringify!($b),
                    ");padding:0 0.7em;margin-right:0.5em;border:1px solid\">",
                    "</span>",
                    "rgb(",
                    stringify!($r),
                    ",",
                    stringify!($g),
                    ",",
                    stringify!($b),
                    ")",
                    "</div>"
                )]
        pub const $name: ColorFormat = rgb($r, $g, $b);
    };
}

colour!(PRIMARY: 107, 83, 140);
colour!(SURFACE_TINT: 107, 83, 140);
colour!(ON_PRIMARY: 255, 255, 255);
colour!(PRIMARY_CONTAINER: 237, 220, 255);
colour!(ON_PRIMARY_CONTAINER: 82, 60, 115);
colour!(SECONDARY: 100, 90, 112);
colour!(ON_SECONDARY: 255, 255, 255);
colour!(SECONDARY_CONTAINER: 235, 221, 247);
colour!(ON_SECONDARY_CONTAINER: 76, 67, 87);
colour!(TERTIARY: 127, 82, 91);
colour!(ON_TERTIARY: 255, 255, 255);
colour!(TERTIARY_CONTAINER: 255, 217, 223);
colour!(ON_TERTIARY_CONTAINER: 101, 59, 67);
colour!(ERROR: 186, 26, 26);
colour!(ON_ERROR: 255, 255, 255);
colour!(ERROR_CONTAINER: 255, 218, 214);
colour!(ON_ERROR_CONTAINER: 147, 0, 10);
colour!(BACKGROUND: 255, 247, 255);
colour!(ON_BACKGROUND: 29, 26, 32);
colour!(SURFACE: 255, 247, 255);
colour!(ON_SURFACE: 29, 26, 32);
colour!(SURFACE_VARIANT: 232, 224, 235);
colour!(ON_SURFACE_VARIANT: 74, 69, 78);
colour!(OUTLINE: 123, 117, 127);
colour!(OUTLINE_VARIANT: 204, 196, 207);
colour!(SHADOW: 0, 0, 0);
colour!(SCRIM: 0, 0, 0);
colour!(INVERSE_SURFACE: 50, 47, 53);
colour!(INVERSE_ON_SURFACE: 246, 238, 246);
colour!(INVERSE_PRIMARY: 214, 187, 251);
colour!(PRIMARY_FIXED: 237, 220, 255);
colour!(ON_PRIMARY_FIXED: 37, 14, 68);
colour!(PRIMARY_FIXED_DIM: 214, 187, 251);
colour!(ON_PRIMARY_FIXED_VARIANT: 82, 60, 115);
colour!(SECONDARY_FIXED: 235, 221, 247);
colour!(ON_SECONDARY_FIXED: 32, 24, 42);
colour!(SECONDARY_FIXED_DIM: 206, 194, 218);
colour!(ON_SECONDARY_FIXED_VARIANT: 76, 67, 87);
colour!(TERTIARY_FIXED: 255, 217, 223);
colour!(ON_TERTIARY_FIXED: 50, 16, 25);
colour!(TERTIARY_FIXED_DIM: 242, 183, 194);
colour!(ON_TERTIARY_FIXED_VARIANT: 101, 59, 67);
colour!(SURFACE_DIM: 223, 216, 224);
colour!(SURFACE_BRIGHT: 255, 247, 255);
colour!(SURFACE_CONTAINER_LOWEST: 255, 255, 255);
colour!(SURFACE_CONTAINER_LOW: 249, 241, 249);
colour!(SURFACE_CONTAINER: 243, 236, 244);
colour!(SURFACE_CONTAINER_HIGH: 237, 230, 238);
colour!(SURFACE_CONTAINER_HIGHEST: 231, 224, 232);
