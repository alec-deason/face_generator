use rand::prelude::*;
use std::collections::HashMap;
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
    let h = h*360.0;
    let c = (1.0 - (2.0*l - 1.0).abs()) * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - ((h2 % 2.0) - 1.0).abs());
    let (r, g, b) = if (0.0 <= h2) & (h2 <= 1.0) { (c, x, 0.0) } else
                    if (1.0 <= h2) & (h2 <= 2.0) { (x, c, 0.0) } else
                    if (2.0 <= h2) & (h2 <= 3.0) { (0.0, c, x) } else
                    if (3.0 <= h2) & (h2 <= 4.0) { (0.0, x, c) } else
                    if (4.0 <= h2) & (h2 <= 5.0) { (x, 0.0, c) } else
                    if (5.0 <= h2) & (h2 <= 6.0) { (c, 0.0, x) } else
                                                 { (0.0, 0.0, 0.0) };
    let m = l - c/2.0;

    let mut rgb = ((r+m) * 255.0) as u32;
    rgb = (rgb << 8) + ((g+m) * 255.0) as u32;
    rgb = (rgb << 8) + ((b+m) * 255.0) as u32;
    rgb
}
