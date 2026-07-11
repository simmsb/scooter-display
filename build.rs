use std::process::Command;

use material_colors::{color::Rgb, dynamic_color::Variant};

const SOURCE_COLOUR: Rgb = Rgb::new(218, 189, 254);
const DARK_MODE: bool = false;

const THEME_FIELDS: &[&str] = &[
    "primary",
    "surface_tint",
    "on_primary",
    "primary_container",
    "on_primary_container",
    "secondary",
    "on_secondary",
    "secondary_container",
    "on_secondary_container",
    "tertiary",
    "on_tertiary",
    "tertiary_container",
    "on_tertiary_container",
    "error",
    "on_error",
    "error_container",
    "on_error_container",
    "background",
    "on_background",
    "surface",
    "on_surface",
    "surface_variant",
    "on_surface_variant",
    "outline",
    "outline_variant",
    "shadow",
    "scrim",
    "inverse_surface",
    "inverse_on_surface",
    "inverse_primary",
    "primary_fixed",
    "on_primary_fixed",
    "primary_fixed_dim",
    "on_primary_fixed_variant",
    "secondary_fixed",
    "on_secondary_fixed",
    "secondary_fixed_dim",
    "on_secondary_fixed_variant",
    "tertiary_fixed",
    "on_tertiary_fixed",
    "tertiary_fixed_dim",
    "on_tertiary_fixed_variant",
    "surface_dim",
    "surface_bright",
    "surface_container_lowest",
    "surface_container_low",
    "surface_container",
    "surface_container_high",
    "surface_container_highest",
];

macro_rules! add {
    ($s:ident, $scheme:ident, $name:ident) => {{
        let Rgb { red, green, blue } = $scheme.$name();
        let name = stringify!($name).to_ascii_uppercase();
        $s.push_str(&format!("colour!({name}: {red}, {green}, {blue});\n"));
    }};
}

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", &git_hash[..8]);

    let variant = &Variant::TonalSpot;
    let scheme = material_colors::dynamic_color::DynamicScheme::by_variant(
        SOURCE_COLOUR,
        variant,
        DARK_MODE,
        None,
    )
    .with_spec_version(material_colors::dynamic_color::color_spec::SpecVersion::Spec2021);

    let mut generated = String::new();

    add!(generated, scheme, primary);
    add!(generated, scheme, surface_tint);
    add!(generated, scheme, on_primary);
    add!(generated, scheme, primary_container);
    add!(generated, scheme, on_primary_container);
    add!(generated, scheme, secondary);
    add!(generated, scheme, on_secondary);
    add!(generated, scheme, secondary_container);
    add!(generated, scheme, on_secondary_container);
    add!(generated, scheme, tertiary);
    add!(generated, scheme, on_tertiary);
    add!(generated, scheme, tertiary_container);
    add!(generated, scheme, on_tertiary_container);
    add!(generated, scheme, error);
    add!(generated, scheme, on_error);
    add!(generated, scheme, error_container);
    add!(generated, scheme, on_error_container);
    add!(generated, scheme, background);
    add!(generated, scheme, on_background);
    add!(generated, scheme, surface);
    add!(generated, scheme, on_surface);
    add!(generated, scheme, surface_variant);
    add!(generated, scheme, on_surface_variant);
    add!(generated, scheme, outline);
    add!(generated, scheme, outline_variant);
    add!(generated, scheme, shadow);
    add!(generated, scheme, scrim);
    add!(generated, scheme, inverse_surface);
    add!(generated, scheme, inverse_on_surface);
    add!(generated, scheme, inverse_primary);
    add!(generated, scheme, primary_fixed);
    add!(generated, scheme, on_primary_fixed);
    add!(generated, scheme, primary_fixed_dim);
    add!(generated, scheme, on_primary_fixed_variant);
    add!(generated, scheme, secondary_fixed);
    add!(generated, scheme, on_secondary_fixed);
    add!(generated, scheme, secondary_fixed_dim);
    add!(generated, scheme, on_secondary_fixed_variant);
    add!(generated, scheme, tertiary_fixed);
    add!(generated, scheme, on_tertiary_fixed);
    add!(generated, scheme, tertiary_fixed_dim);
    add!(generated, scheme, on_tertiary_fixed_variant);
    add!(generated, scheme, surface_dim);
    add!(generated, scheme, surface_bright);
    add!(generated, scheme, surface_container_lowest);
    add!(generated, scheme, surface_container_low);
    add!(generated, scheme, surface_container);
    add!(generated, scheme, surface_container_high);
    add!(generated, scheme, surface_container_highest);

    build_script_file_gen::gen_file_str("generated_colours.rs", &generated);

    let mut theme_impl = String::from(
        "Self {\n            hue: settings.hue,\n            saturation: settings.saturation,\n            value: settings.value,\n            dark_mode: settings.dark_mode,\n",
    );
    for field in THEME_FIELDS {
        theme_impl.push_str(&format!("            {field}: to_rgb(scheme.{field}()),\n"));
    }
    theme_impl.push_str("        }");
    build_script_file_gen::gen_file_str("generated_theme_impl.rs", &theme_impl);
}
