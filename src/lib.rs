use rand::{Rng};
use rand::seq::SliceRandom;


use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub struct FaceGenerator {
    assets_dir: PathBuf,
    available_assets: HashMap<String, Vec<u32>>,
}

impl FaceGenerator {
    pub fn new(assets: &Path) -> FaceGenerator {
        let mut g = FaceGenerator {
            assets_dir: assets.to_path_buf(),
            available_assets: HashMap::new(),
        };
        g.check_assets().unwrap();
        g
    }

    fn check_assets(&mut self) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(&self.assets_dir)? {
            let entry = entry?;
            let path = entry.path();
            let mut ids:Vec<u32> = Vec::new();
            if path.is_dir() {
                for asset_entry in fs::read_dir(path)? {
                    let asset_entry = asset_entry?;
                    let file_name = asset_entry.file_name();
                    let name = Path::new(&file_name).file_stem().unwrap();
                    match name.to_str() {
                        Some(name) => match name.parse::<u32>() {
                            Ok(id) => ids.push(id),
                            Err(_) => (),
                        },
                        None => (),
                    } 
                }
                let asset_name = entry.path().file_stem().unwrap().to_str().unwrap().to_string();
                self.available_assets.insert(asset_name, ids);
            }
        }

        Ok(())
    }

    pub fn generate(&self) -> Face {
        let mut rng = rand::thread_rng();

        Face {
            face: *self.available_assets["face"].choose(&mut rng).unwrap(),
            ears: *self.available_assets["ears"].choose(&mut rng).unwrap(),
            eyes: *self.available_assets["eyes"].choose(&mut rng).unwrap(),
            nose: *self.available_assets["nose"].choose(&mut rng).unwrap(),
            mouth: *self.available_assets["mouth"].choose(&mut rng).unwrap(),
            hair: *self.available_assets["hair"].choose(&mut rng).unwrap(),
            pallet: self.select_pallet(),
        }

    }

    fn select_pallet(&self) -> &'static Pallet {
        let choice = rand::thread_rng().gen_range(0, PALLETS.len());
        &PALLETS[choice]
    }

    pub fn to_svg_fragment(&self, face: &Face) -> Result<SVGFragment, std::io::Error> {
        let mut contents = String::new();

        contents.push_str(&self.asset_to_string("ears", face.ears)?);
        contents.push_str(&self.asset_to_string("face", face.face)?);
        contents.push_str(&self.asset_to_string("eyes", face.eyes)?);
        contents.push_str(&self.asset_to_string("mouth", face.mouth)?);
        contents.push_str(&self.asset_to_string("nose", face.nose)?);
        contents.push_str(&self.asset_to_string("hair", face.hair)?);


        for (a, b) in face.pallet {
            let pattern = format!("fill:{};", a);
            let replacement = format!("fill:{};", b);
            contents = contents.replace(&pattern, &replacement);
        }

        Ok(SVGFragment {
            contents: contents,
        })
    }

    fn asset_to_string(&self, asset: &str, id: u32) -> Result<String, std::io::Error> {
        let asset_file = format!("{}.svg", id);
        let mut file = File::open(self.assets_dir.join(asset).join(asset_file))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        Ok(contents)
    }
}

pub struct SVGFragment {
    contents: String,
}

impl SVGFragment {
    fn to_string(&self, w: f64, x: f64, y: f64) -> String {
        let mut group = format!("<svg x='{}px' y='{}px' width='{}px' viewBox='0 0 210 210'>", x, y, w).to_string();

        group.push_str(&self.contents);

        group.push_str("</svg>");
        group
    }
}

pub fn svg_grid(fragments: &Vec<SVGFragment>, width: u32) -> String {
        let mut doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>

<svg
xmlns:svg="http://www.w3.org/2000/svg"
xmlns="http://www.w3.org/2000/svg"
width="1024px"
height="1024px"
viewBox="0 0 210 210"
version="1.1"
>
        "#.to_owned();

        let stride = 210.0 / width as f64;
        let mut vertical_offset = -90.0;
        let mut horizontal_offset = 0.0;
        let mut in_this_row = 0;
        for frag in fragments {
            doc.push_str(&frag.to_string(stride, horizontal_offset, vertical_offset));
            horizontal_offset += stride;
            in_this_row += 1;
            if in_this_row >= width {
                horizontal_offset = 0.0;
                vertical_offset += stride;
                in_this_row = 0;
            }
        }

        doc.push_str("</svg>");
        doc
}

pub struct Face {
    face: u32,
    ears: u32,
    eyes: u32,
    nose: u32,
    mouth: u32,
    hair: u32,
    pallet: &'static Pallet,
}

type Pallet = [(&'static str, &'static str); 11];
static PALLETS: [Pallet; 2] = [
    [ // Dark tone
            ("skin_color_0", "#df9241ff"),
            ("skin_color_1", "#cb7922ff"),
            ("skin_color_2", "#b66f24ff"),
            ("skin_color_3", "#955c20ff"),
            ("skin_color_4", "#724b21ff"),

            ("eye_color_1", "#f2f2f2ff"),
            ("eye_color_2", "#362512ff"),
            ("eye_color_3", "#0e0202ff"),

            ("hair_color_1", "#1a0f03ff"),
            ("hair_color_2", "#311b04ff"),
            ("hair_color_3", "#4d2c0aff"),
    ],
    [ // Pale tone
            ("skin_color_0", "#e8b79dff"),
            ("skin_color_1", "#e8a279ff"),
            ("skin_color_2", "#eaae70ff"),
            ("skin_color_3", "#df9241ff"),
            ("skin_color_4", "#bc7225ff"),

            ("eye_color_1", "#f2f2f2ff"),
            ("eye_color_2", "#5f8dd3ff"),
            ("eye_color_3", "#00112bff"),

            ("hair_color_1", "#dfb012ff"),
            ("hair_color_2", "#f0ca4aff"),
            ("hair_color_3", "#f5da82ff"),
    ],
];
