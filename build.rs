use material_colors::{color::Rgb, dynamic_color::Variant};

const SOURCE_COLOUR: Rgb = Rgb::new(218, 189, 254);

macro_rules! add {
    ($s:ident, $scheme:ident, $name:ident) => {{
        let Rgb { red, green, blue } = $scheme.$name();
        let name = stringify!($name).to_ascii_uppercase();
        $s.push_str(&format!("colour!({name}: {red}, {green}, {blue});\n"));
    }};
}

fn main() {
    let variant = &Variant::TonalSpot;
    let scheme = material_colors::dynamic_color::DynamicScheme::by_variant(
        SOURCE_COLOUR,
        variant,
        false,
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
}
