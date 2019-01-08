use rand::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::Palette;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ColorComponent {
    Range(f64, f64),
    Constant(f64),
}
type RawPaletteVarient = (ColorComponent, ColorComponent, ColorComponent);
type RawPalette = HashMap<String, HashMap<String, HashMap<String, RawPaletteVarient>>>;

fn sample_component(p: &ColorComponent) -> f64 {
    match p {
        ColorComponent::Range(start, end) => {
            let mut rng = rand::thread_rng();
            rng.gen_range(start, end)
        }
        ColorComponent::Constant(value) => *value,
    }
}

pub fn palette_from_file(path: &Path) -> Palette {
    let mut rng = rand::thread_rng();
    let mut palette = HashMap::new();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let raw_palette: RawPalette = serde_json::from_reader(reader).unwrap();

    // Skin
    let (skin_variant, config) = raw_palette["skin"]["default"]
        .iter()
        .choose(&mut rng)
        .unwrap();
    let h = sample_component(&config.0);
    let s = sample_component(&config.1);
    let l = sample_component(&config.2);
    let rgb = hsl_to_rgb(h, s, l);
    palette.insert("skin_color".to_string(), format!("#{:01$x}", rgb, 6));
    let rgb = hsl_to_rgb(h, s, l * 0.6);
    palette.insert(
        "skin_color_outline".to_string(),
        format!("#{:01$x}", rgb, 6),
    );

    // Hair
    let choices = if skin_variant == "dark" {
        vec!["black", "grey", "brown"]
    } else {
        vec!["black", "grey", "brown", "red", "blond"]
    };
    let variant = choices.choose(&mut rng).unwrap();
    let config = &raw_palette["hair"]["default"][&variant.to_string()];
    let h = sample_component(&config.0);
    let s = sample_component(&config.1);
    let l = sample_component(&config.2);
    let rgb = hsl_to_rgb(h, s, l);
    palette.insert("hair_color".to_string(), format!("#{:01$x}", rgb, 6));
    let rgb = hsl_to_rgb(h, s, l * 0.6);
    palette.insert(
        "hair_color_outline".to_string(),
        format!("#{:01$x}", rgb, 6),
    );

    // Eyes
    let choices = if skin_variant == "dark" {
        vec!["dark_brown", "hazel"]
    } else {
        vec!["dark_brown", "hazel", "blue", "green"]
    };
    let variant = choices.choose(&mut rng).unwrap();
    let config = &raw_palette["eye"]["default"][&variant.to_string()];
    let h = sample_component(&config.0);
    let s = sample_component(&config.1);
    let l = sample_component(&config.2);
    let rgb = hsl_to_rgb(h, s, l);
    palette.insert("eye_color".to_string(), format!("#{:01$x}", rgb, 6));
    let rgb = hsl_to_rgb(h, s, l * 0.6);
    palette.insert("eye_color_outline".to_string(), format!("#{:01$x}", rgb, 6));

    palette
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> u32 {
    let (r, g, b) = if s == 0.0 {
        // achromatic
        (l, l, l)
    } else {
        let hue2rgb = |p, q, mut t| {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.9;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0 / 2.0 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        (
            hue2rgb(p, q, h + 1.0 / 3.0),
            hue2rgb(p, q, h),
            hue2rgb(p, q, h - 1.0 / 3.0),
        )
    };

    let mut rgb = (r * 255.0) as u32;
    rgb = (rgb << 8) + (g * 255.0) as u32;
    rgb = (rgb << 8) + (b * 255.0) as u32;
    rgb
}
