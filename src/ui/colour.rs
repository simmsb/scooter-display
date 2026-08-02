use embedded_graphics::prelude::RgbColor;

#[cfg(feature = "sim")]
pub mod theme;

pub type ColorFormat = embedded_graphics::pixelcolor::Rgb565;

pub const BLACK: ColorFormat = ColorFormat::new(0, 0, 0);

/// Rgb565::new expects inputs to be in the range of the channel, not 8 bits.
/// So we need to scale them.
pub(crate) const fn rgb(r: u8, g: u8, b: u8) -> ColorFormat {
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

include!(concat!(env!("OUT_DIR"), "/", "generated_colours.rs"));

colour!(GREEN: 0, 200, 0);
colour!(ON_GREEN: 255, 255, 255);

#[cfg(feature = "sim")]
pub use theme::{ThemeSettings, set_theme_settings, theme_settings};
