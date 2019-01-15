use rand::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::iter::FromIterator;

use palette::{LinSrgb, LinSrgba, Hsla, Hsl, Color, Shade, Saturate, Pixel, Blend};


use super::Palette;

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(untagged)]
enum ColorComponent {
    Range(f32, f32),
    Constant(f32),
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(untagged)]
enum ColorFunction {
    HSL(ColorComponent, ColorComponent, ColorComponent),
    SkinModel((f32, f32), (f32, f32, f32), (f32, f32, f32)),
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum RawPaletteVarient {
    Color(ColorFunction),
    ColorWithSubchoice(ColorFunction, HashMap<String, Vec<String>>),
}
type RawPalette = Vec<(String, HashMap<String, HashMap<String, RawPaletteVarient>>)>;

fn sample_component(p: &ColorComponent) -> f32 {
    match p {
        ColorComponent::Range(start, end) => {
            let mut rng = rand::thread_rng();
            rng.gen_range(start, end)
        }
        ColorComponent::Constant(value) => *value,
    }
}

fn rec_choose_variant(palette: &HashMap<String, HashMap<String, HashMap<String, RawPaletteVarient>>>, palette_type: &str, section: &String, constraints: Option<&Vec<&String>>, values_chosen: &mut HashMap<String, (String, Color)>) {
    if !values_chosen.contains_key(section) {
        let mut rng = rand::thread_rng();
        let sub_section = &palette[section];
        let sub_section = if sub_section.contains_key(palette_type) {
            &sub_section[palette_type]
        } else {
            &sub_section["default"]
        };
        let options = match constraints {
            Some(constraints) => constraints.clone(),
            None => sub_section.keys().collect(),
        };
        let variant = options.iter().choose(&mut rng).unwrap();
        let config = &sub_section[*variant];
        let color_function = match config {
            RawPaletteVarient::Color(func) => func,
            RawPaletteVarient::ColorWithSubchoice(func, sub_choices) => {
                for (section, constraints) in sub_choices.iter() {
                    rec_choose_variant(palette, palette_type, section, Some(&constraints.iter().collect()), values_chosen);
                }
                func
            },
        };
        let color = match color_function {
            ColorFunction::HSL(h, s, l) => {
                let h = sample_component(&h);
                let s = sample_component(&s);
                let l = sample_component(&l);
                Color::Hsl(Hsl::new(h * 360.0, s, l))
            },
            ColorFunction::SkinModel((alpha_start, alpha_end), (ph, ps, pl), (bh, bs, bl)) =>
            {
                let mut rng = rand::thread_rng();
                let a1 = rng.gen_range(alpha_start, alpha_end);
                // FIXME: If I use Hsla directly it ignores alpha when I composite
                // probably a bug in palette?
                let p1 = LinSrgba::from(Hsla::new(*ph, *ps/100.0, *pl/100.0, a1));
                let b = LinSrgba::from(Hsla::new(*bh, *bs/100.0, *bl/100.0, 1.0));

                Color::from(p1.over(b))
            },
        };
        values_chosen.insert(section.to_string(), (variant.to_string(), color));
    }
}

fn rgb_to_svg(rgb: &LinSrgb) -> String {
    let mut rgb_int = (rgb.red * 256.0) as u32;
    rgb_int = (rgb_int << 8) + (rgb.green * 256.0) as u32;
    rgb_int = (rgb_int << 8) + (rgb.blue * 256.0) as u32;
    format!("#{:01$x}", rgb_int, 6)
}

pub fn palette_from_file(path: &Path, palette_type: &str) -> (String, Palette) {
    let mut palette = HashMap::new();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let raw_palette: RawPalette = serde_json::from_reader(reader).unwrap();
    let sections:Vec<String> = raw_palette.iter().map(|(k,v)| k.to_string()).collect();
    let raw_palette:HashMap<String, HashMap<String, HashMap<String, RawPaletteVarient>>> = HashMap::from_iter(raw_palette);

    let mut values_chosen = HashMap::new();
    for section in sections {
        rec_choose_variant(&raw_palette, palette_type, &section, None, &mut values_chosen);
    }

    for (section, (variant, color)) in values_chosen.iter() {
        let hsl = Hsl::from(*color);
        let rgb = LinSrgb::from(hsl);
        palette.insert(section.to_string(), rgb_to_svg(&rgb));
        let rgb = LinSrgb::from(
            hsl.darken(hsl.lightness - hsl.lightness * 0.6).
            desaturate(hsl.saturation - hsl.saturation * 0.6)
        );
        palette.insert(
            format!("{}_outline", section),
            rgb_to_svg(&rgb),
        );
    }
    let palette_path:Vec<String> = values_chosen.iter().map(|(k,v)| format!("{}:{}", k, v.0)).collect();
    let palette_path = format!("{}:{}", palette_type, palette_path.join(":"));
    (palette_path, palette)
}
