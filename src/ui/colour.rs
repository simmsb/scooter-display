use embedded_graphics::prelude::RgbColor;

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

pub mod theme;

macro_rules! colour_accessors {
    ($($name:ident),* $(,)?) => {
        $(
            #[cfg(not(feature = "sim"))]
            #[inline(always)]
            pub const fn $name() -> ColorFormat {
                paste::paste! { [<$name:upper>] }
            }

            #[cfg(feature = "sim")]
            pub fn $name() -> ColorFormat {
                theme::with_theme(|theme| theme.$name)
            }
        )*
    };
}

colour_accessors! {
    primary,
    surface_tint,
    on_primary,
    primary_container,
    on_primary_container,
    secondary,
    on_secondary,
    secondary_container,
    on_secondary_container,
    tertiary,
    on_tertiary,
    tertiary_container,
    on_tertiary_container,
    error,
    on_error,
    error_container,
    on_error_container,
    background,
    on_background,
    surface,
    on_surface,
    surface_variant,
    on_surface_variant,
    outline,
    outline_variant,
    shadow,
    scrim,
    inverse_surface,
    inverse_on_surface,
    inverse_primary,
    primary_fixed,
    on_primary_fixed,
    primary_fixed_dim,
    on_primary_fixed_variant,
    secondary_fixed,
    on_secondary_fixed,
    secondary_fixed_dim,
    on_secondary_fixed_variant,
    tertiary_fixed,
    on_tertiary_fixed,
    tertiary_fixed_dim,
    on_tertiary_fixed_variant,
    surface_dim,
    surface_bright,
    surface_container_lowest,
    surface_container_low,
    surface_container,
    surface_container_high,
    surface_container_highest,
}

#[cfg(feature = "sim")]
pub use theme::{ThemeSettings, set_theme_settings, theme_settings};

pub const fn green() -> ColorFormat {
    GREEN
}

pub const fn on_green() -> ColorFormat {
    ON_GREEN
}

pub const fn black() -> ColorFormat {
    BLACK
}
