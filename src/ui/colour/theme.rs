use super::ColorFormat;

pub const DEFAULT_SEED: (u8, u8, u8) = (218, 189, 254);

#[cfg(feature = "sim")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeSettings {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub dark_mode: bool,
}

pub struct Theme {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub dark_mode: bool,
    pub primary: ColorFormat,
    pub surface_tint: ColorFormat,
    pub on_primary: ColorFormat,
    pub primary_container: ColorFormat,
    pub on_primary_container: ColorFormat,
    pub secondary: ColorFormat,
    pub on_secondary: ColorFormat,
    pub secondary_container: ColorFormat,
    pub on_secondary_container: ColorFormat,
    pub tertiary: ColorFormat,
    pub on_tertiary: ColorFormat,
    pub tertiary_container: ColorFormat,
    pub on_tertiary_container: ColorFormat,
    pub error: ColorFormat,
    pub on_error: ColorFormat,
    pub error_container: ColorFormat,
    pub on_error_container: ColorFormat,
    pub background: ColorFormat,
    pub on_background: ColorFormat,
    pub surface: ColorFormat,
    pub on_surface: ColorFormat,
    pub surface_variant: ColorFormat,
    pub on_surface_variant: ColorFormat,
    pub outline: ColorFormat,
    pub outline_variant: ColorFormat,
    pub shadow: ColorFormat,
    pub scrim: ColorFormat,
    pub inverse_surface: ColorFormat,
    pub inverse_on_surface: ColorFormat,
    pub inverse_primary: ColorFormat,
    pub primary_fixed: ColorFormat,
    pub on_primary_fixed: ColorFormat,
    pub primary_fixed_dim: ColorFormat,
    pub on_primary_fixed_variant: ColorFormat,
    pub secondary_fixed: ColorFormat,
    pub on_secondary_fixed: ColorFormat,
    pub secondary_fixed_dim: ColorFormat,
    pub on_secondary_fixed_variant: ColorFormat,
    pub tertiary_fixed: ColorFormat,
    pub on_tertiary_fixed: ColorFormat,
    pub tertiary_fixed_dim: ColorFormat,
    pub on_tertiary_fixed_variant: ColorFormat,
    pub surface_dim: ColorFormat,
    pub surface_bright: ColorFormat,
    pub surface_container_lowest: ColorFormat,
    pub surface_container_low: ColorFormat,
    pub surface_container: ColorFormat,
    pub surface_container_high: ColorFormat,
    pub surface_container_highest: ColorFormat,
}

#[cfg(feature = "sim")]
mod runtime {
    use std::sync::Mutex;

    use material_colors::{color::Rgb, dynamic_color::Variant};
    use once_cell::sync::Lazy;

    use super::{ColorFormat, DEFAULT_SEED, Theme, ThemeSettings};

    static THEME: Lazy<Mutex<Theme>> = Lazy::new(|| {
        let (hue, saturation, value) = hsv_from_rgb(DEFAULT_SEED);
        Mutex::new(Theme::from_settings(ThemeSettings {
            hue,
            saturation,
            value,
            dark_mode: false,
        }))
    });

    pub fn with_theme<R>(f: impl FnOnce(&Theme) -> R) -> R {
        let theme = THEME.lock().unwrap();
        f(&theme)
    }

    pub fn theme_settings() -> ThemeSettings {
        with_theme(|theme| ThemeSettings {
            hue: theme.hue,
            saturation: theme.saturation,
            value: theme.value,
            dark_mode: theme.dark_mode,
        })
    }

    pub fn set_theme_settings(settings: ThemeSettings) -> bool {
        let mut theme = THEME.lock().unwrap();
        if theme.hue == settings.hue
            && (theme.saturation - settings.saturation).abs() < 0.001
            && (theme.value - settings.value).abs() < 0.001
            && theme.dark_mode == settings.dark_mode
        {
            return false;
        }
        *theme = Theme::from_settings(settings);
        true
    }

    impl Theme {
        pub fn from_settings(settings: ThemeSettings) -> Self {
            let (red, green, blue) = hsv_to_rgb(settings.hue, settings.saturation, settings.value);
            let scheme = material_colors::dynamic_color::DynamicScheme::by_variant(
                Rgb::new(red, green, blue),
                &Variant::TonalSpot,
                settings.dark_mode,
                None,
            )
            .with_spec_version(material_colors::dynamic_color::color_spec::SpecVersion::Spec2021);

            Self::from_material_scheme(&scheme, settings)
        }

        fn from_material_scheme(
            scheme: &material_colors::dynamic_color::DynamicScheme,
            settings: ThemeSettings,
        ) -> Self {
            #[allow(clippy::needless_return)]
            return include!(concat!(env!("OUT_DIR"), "/generated_theme_impl.rs"));
        }
    }

    fn hsv_from_rgb((red, green, blue): (u8, u8, u8)) -> (f32, f32, f32) {
        let (red, green, blue) = (
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
        );
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let delta = max - min;

        let hue = if delta < f32::EPSILON {
            0.0
        } else if (max - red).abs() < f32::EPSILON {
            let h = 60.0 * (((green - blue) / delta) % 6.0);
            if h < 0.0 { h + 360.0 } else { h }
        } else if (max - green).abs() < f32::EPSILON {
            60.0 * (((blue - red) / delta) + 2.0)
        } else {
            60.0 * (((red - green) / delta) + 4.0)
        };

        let saturation = if max < f32::EPSILON { 0.0 } else { delta / max };

        (hue, saturation, max)
    }

    fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
        let chroma = value * saturation;
        let hue_prime = hue / 60.0;
        let x = chroma * (1.0 - ((hue_prime % 2.0) - 1.0).abs());
        let (red1, green1, blue1) = if hue_prime < 1.0 {
            (chroma, x, 0.0)
        } else if hue_prime < 2.0 {
            (x, chroma, 0.0)
        } else if hue_prime < 3.0 {
            (0.0, chroma, x)
        } else if hue_prime < 4.0 {
            (0.0, x, chroma)
        } else if hue_prime < 5.0 {
            (x, 0.0, chroma)
        } else {
            (chroma, 0.0, x)
        };
        let match_value = value - chroma;
        let to_u8 =
            |channel: f32| ((channel + match_value) * 255.0).round().clamp(0.0, 255.0) as u8;
        (to_u8(red1), to_u8(green1), to_u8(blue1))
    }

    pub(crate) fn to_rgb(color: Rgb) -> ColorFormat {
        super::super::rgb(color.red, color.green, color.blue)
    }
}

#[cfg(feature = "sim")]
pub use runtime::{set_theme_settings, theme_settings, with_theme};
