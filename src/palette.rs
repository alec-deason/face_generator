use rand::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::iter::FromIterator;

use super::Palette;

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(untagged)]
enum ColorComponent {
    Range(f64, f64),
    Constant(f64),
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum RawPaletteVarient {
    Color(ColorComponent, ColorComponent, ColorComponent),
    ColorWithSubchoice(ColorComponent, ColorComponent, ColorComponent, HashMap<String, Vec<String>>),
}
type RawPalette = Vec<(String, HashMap<String, HashMap<String, RawPaletteVarient>>)>;

fn sample_component(p: &ColorComponent) -> f64 {
    match p {
        ColorComponent::Range(start, end) => {
            let mut rng = rand::thread_rng();
            rng.gen_range(start, end)
        }
        ColorComponent::Constant(value) => *value,
    }
}

fn rec_choose_variant(palette: &HashMap<String, HashMap<String, HashMap<String, RawPaletteVarient>>>, section: &String, constraints: Option<&Vec<&String>>, values_chosen: &mut HashMap<String, (String, (ColorComponent, ColorComponent, ColorComponent))>) {
    if !values_chosen.contains_key(section) {
        let mut rng = rand::thread_rng();
        let sub_section = &palette[section]["default"];
        let options = match constraints {
            Some(constraints) => constraints.clone(),
            None => sub_section.keys().collect(),
        };
        let variant = options.iter().choose(&mut rng).unwrap();
        let config = &sub_section[*variant];
        let (h, s, l) = match config {
            RawPaletteVarient::Color(h, s, l) => (h,s,l),
            RawPaletteVarient::ColorWithSubchoice(h, s, l, sub_choices) => {
                for (section, constraints) in sub_choices.iter() {
                    rec_choose_variant(palette, section, Some(&constraints.iter().collect()), values_chosen);
                }
                (h,s,l)
            },
        };
        values_chosen.insert(section.to_string(), (variant.to_string(), (*h, *s, *l)));
    }
}

pub fn palette_from_file(path: &Path) -> (String, Palette) {
    let mut palette = HashMap::new();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let raw_palette: RawPalette = serde_json::from_reader(reader).unwrap();
    let sections:Vec<String> = raw_palette.iter().map(|(k,v)| k.to_string()).collect();
    let raw_palette:HashMap<String, HashMap<String, HashMap<String, RawPaletteVarient>>> = HashMap::from_iter(raw_palette);

    let mut values_chosen = HashMap::new();
    for section in sections {
        rec_choose_variant(&raw_palette, &section, None, &mut values_chosen);
    }

    for (section, (variant, (h, s, l))) in values_chosen.iter() {
        let h = sample_component(&h);
        let s = sample_component(&s);
        let l = sample_component(&l);
        let rgb = hsl_to_rgb(h, s, l);
        palette.insert(section.to_string(), format!("#{:01$x}", rgb, 6));
        let rgb = hsl_to_rgb(h, s, l * 0.6);
        palette.insert(
            format!("{}_outline", section),
            format!("#{:01$x}", rgb, 6),
        );
    }
    let palette_path:Vec<String> = values_chosen.iter().map(|(k,v)| format!("{}:{}", k, v.0)).collect();
    let palette_path = palette_path.join(":");
    (palette_path, palette)
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