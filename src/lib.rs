use rand::{Rng};
use rand::seq::SliceRandom;
use rand::seq::IteratorRandom;


use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub struct FaceGenerator {
    assets_dir: PathBuf,
    available_assets: HashMap<String, HashMap<u32, bool>>,
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
            let mut ids:HashMap<u32, bool> = HashMap::with_capacity(5);
            if path.is_dir() {
                for asset_entry in fs::read_dir(path)? {
                    let asset_entry = asset_entry?;
                    let file_name = asset_entry.file_name();
                    let name = Path::new(&file_name).file_stem().unwrap();
                    match name.to_str() {
                        Some(name) => {
                            let real_name;
                            let is_back;
                            if name.contains("_back") {
                                real_name = &name[..name.len()-5];
                                is_back = true;
                            } else {
                                real_name = &name;
                                is_back = false;
                            }
                            match real_name.parse::<u32>() {
                                Ok(id) => {
                                    ids.entry(id)
                                    .and_modify(|e| *e = *e | is_back)
                                    .or_insert(is_back);
                                },
                                Err(_) => (),
                            }
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
            face: *self.available_assets["face"].keys().into_iter().choose(&mut rng).unwrap(),
            ears: *self.available_assets["ears"].keys().into_iter().choose(&mut rng).unwrap(),
            eyes: *self.available_assets["eyes"].keys().into_iter().choose(&mut rng).unwrap(),
            eyebrows: *self.available_assets["eyebrows"].keys().into_iter().choose(&mut rng).unwrap(),
            nose: *self.available_assets["nose"].keys().into_iter().choose(&mut rng).unwrap(),
            mouth: *self.available_assets["mouth"].keys().into_iter().choose(&mut rng).unwrap(),
            hair: *self.available_assets["hair"].keys().into_iter().choose(&mut rng).unwrap(),
            pallet: self.select_pallet(),
        }

    }

    fn select_pallet(&self) -> Pallet {
        let mut rng = rand::thread_rng();

        let base_skin_tone = (
            rng.gen_range(21.0, 35.0),
            (163.0 / 256.0) * 100.0,
            (rng.gen_range(97.0, 156.0) / 256.0) * 100.0,
        );

        let mut pallet = HashMap::new();

        pallet.insert("skin_color_0".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_skin_tone.0, base_skin_tone.1, base_skin_tone.2 * 1.34));
        pallet.insert("skin_color_1".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_skin_tone.0, base_skin_tone.1, base_skin_tone.2 * 1.12));
        pallet.insert("skin_color_2".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_skin_tone.0, base_skin_tone.1, base_skin_tone.2));
        pallet.insert("skin_color_3".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_skin_tone.0, base_skin_tone.1, base_skin_tone.2 * 0.81));
        pallet.insert("skin_color_4".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_skin_tone.0, base_skin_tone.1, base_skin_tone.2 * 0.5));

        let is_pale_complexion = base_skin_tone.2 > (120.0 / 256.0) * 100.0;

        // TODO: Martin scale!
        let base_eye_color;
        match rng.gen_range(0, if is_pale_complexion { 4 } else { 2 }) {
            0 => { // Dark brown
                base_eye_color = (
                    rng.gen_range(18.0, 27.0),
                    (222.0 / 256.0) * 100.0,
                    (rng.gen_range(50.0, 82.0) / 256.0) * 100.0,
                );
            },
            2 => { // Hazel
                base_eye_color = (
                    rng.gen_range(18.0, 27.0),
                    (222.0 / 256.0) * 100.0,
                    (rng.gen_range(60.0, 92.0) / 256.0) * 100.0,
                );
            },
            3 => { // Blue
                base_eye_color = (
                    rng.gen_range(150.0, 160.0),
                    (161.0 / 256.0) * 100.0,
                    (rng.gen_range(104.0, 160.0) / 256.0) * 100.0,
                );
            },
            _ => { // Green
                base_eye_color = (
                    rng.gen_range(70.0, 90.0),
                    (161.0 / 256.0) * 100.0,
                    (rng.gen_range(104.0, 160.0) / 256.0) * 100.0,
                );
            },
        }

        pallet.insert("eye_color_1".to_string(), "#f2f2f2ff".to_string());
        pallet.insert("eye_color_2".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_eye_color.0, base_eye_color.1, base_eye_color.2));
        pallet.insert("eye_color_3".to_string(), "#00112bff".to_string());

        let base_hair_color;
        match rng.gen_range(0, if is_pale_complexion { 4 } else { 2 }) {
            0 => { // Black Hair
                base_hair_color = (
                    rng.gen_range(18.0, 27.0),
                    (222.0 / 256.0) * 100.0,
                    (rng.gen_range(4.0, 20.0) / 256.0) * 100.0,
                );
            },
            1 => { // Brown Hair
                base_hair_color = (
                    rng.gen_range(18.0, 27.0),
                    (222.0 / 256.0) * 100.0,
                    (rng.gen_range(50.0, 82.0) / 256.0) * 100.0,
                );
            },
            2 => { // Red hair
                base_hair_color = (
                    rng.gen_range(6.0, 15.0),
                    (222.0 / 256.0) * 100.0,
                    (rng.gen_range(100.0, 140.0) / 256.0) * 100.0,
                );
            },
            _ => { // Blond hair
                base_hair_color = (
                    rng.gen_range(29.0, 40.0),
                    (222.0 / 256.0) * 100.0,
                    (rng.gen_range(100.0, 150.0) / 256.0) * 100.0,
                );
            },
        }

        pallet.insert("hair_color_1".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_hair_color.0, base_hair_color.1, base_hair_color.2 * 1.2));
        pallet.insert("hair_color_2".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_hair_color.0, base_hair_color.1, base_hair_color.2));
        pallet.insert("hair_color_3".to_string(), format!("hsl({}, {:.2}%, {:.2}%)", base_hair_color.0, base_hair_color.1, base_hair_color.2 * 0.8));

        pallet
    }

    pub fn to_svg_fragment(&self, face: &Face) -> Result<SVGFragment, std::io::Error> {
        let mut contents = String::new();

        if self.available_assets["hair"][&face.hair] {
            contents.push_str(&self.asset_to_string("hair", face.hair, true)?);
        }
        contents.push_str(&self.asset_to_string("ears", face.ears, false)?);
        contents.push_str(&self.asset_to_string("face", face.face, false)?);
        contents.push_str(&self.asset_to_string("eyes", face.eyes, false)?);
        contents.push_str(&self.asset_to_string("eyebrows", face.eyebrows, false)?);
        contents.push_str(&self.asset_to_string("mouth", face.mouth, false)?);
        contents.push_str(&self.asset_to_string("nose", face.nose, false)?);
        contents.push_str(&self.asset_to_string("hair", face.hair, false)?);


        for (a, b) in &face.pallet {
            let pattern = format!("fill:{};", a);
            let replacement = format!("fill:{};", b);
            contents = contents.replace(&pattern, &replacement);

            let pattern = format!("stroke:{};", a);
            let replacement = format!("stroke:{};", b);
            contents = contents.replace(&pattern, &replacement);
        }

        Ok(SVGFragment {
            contents: contents,
        })
    }

    fn asset_to_string(&self, asset: &str, id: u32, back: bool) -> Result<String, std::io::Error> {
        let suffix;
        if back {
            suffix = "_back";
        } else {
            suffix = "";
        }
        let asset_file = format!("{}{}.svg", id, suffix);
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
    eyebrows: u32,
    nose: u32,
    mouth: u32,
    hair: u32,
    pallet: Pallet,
}

type Pallet = HashMap<String, String>;
