extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate svgtypes;

use rand::{Rng};
use rand::seq::SliceRandom;
use rand::seq::IteratorRandom;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

mod eyes;
mod face;
mod placeable_feature;

pub struct FaceGenerator {
    assets_dir: PathBuf,
    available_assets: HashMap<String, AbstractAsset>,
    skulls: Vec<(String, Skull)>,
}

trait AbstractAssetTrait {
    fn choose(&self, skull: &Skull) -> ConcreteAsset;
}
type AbstractAsset = Box<AbstractAssetTrait>;

trait ConcreteAssetTrait {
    fn to_svg_fragments(&self) -> Vec<SVGFragment>;
}
type ConcreteAsset = Box<ConcreteAssetTrait>;

impl ConcreteAssetTrait for (SVGFragment, Option<SVGFragment>) {
    fn to_svg_fragments(&self) -> Vec<SVGFragment> {
        let mut result = Vec::with_capacity(2);
        result.push(self.0.clone());
        if self.1.is_some(){
            result.push(self.1.clone().unwrap());
        }
        result
    }
}

struct FileBackedAsset {
    dir: PathBuf,
    ids: Vec<(u32, bool)>,
    front_layer: u32,
    back_layer: u32,
}

impl AbstractAssetTrait for FileBackedAsset {
    fn choose(&self, _skull: &Skull) -> ConcreteAsset {
        let mut rng = rand::thread_rng();

        let (id, has_back) = self.ids.choose(&mut rng).unwrap();
        let load_str = |id, back| {
            let suffix;
            if back {
                suffix = "_back";
            } else {
                suffix = "";
            }
            let asset_file = format!("{}{}.svg", id, suffix);
            let mut file = File::open(self.dir.join(asset_file)).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents);
            contents
        };

        Box::new((
            SVGFragment {
                contents: load_str(id, false),
                layer: self.front_layer,
            },
            if *has_back {
                Some(
                    SVGFragment {
                        contents: load_str(id, true),
                        layer: self.back_layer,
                    }
                )
            } else { None }
        ))
    }
}

impl FaceGenerator {
    pub fn new(assets: &Path) -> FaceGenerator {

        let mut g = FaceGenerator {
            assets_dir: assets.to_path_buf(),
            available_assets: HashMap::new(),
            skulls: Vec::new(),
        };
        g.check_assets().unwrap();
        g
    }

    fn load_skulls(&mut self) -> Result<(), std::io::Error> {
        let skulls_dir = self.assets_dir.join("skulls");
        for entry in fs::read_dir(skulls_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut file = File::open(path.join("skull.json"))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let skull:Skull = serde_json::from_str(&contents).unwrap();
                let file_name = entry.file_name();
                let name = Path::new(&file_name).file_stem().unwrap();
                self.skulls.push((name.to_str().unwrap().to_string(), skull));
            }
        }

        Ok(())
    }

    fn check_assets(&mut self) -> Result<(), std::io::Error> {
        self.load_skulls()?;

        for entry in fs::read_dir(&self.assets_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut ids:HashMap<u32, bool> = HashMap::with_capacity(5);
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
                let mut ids_vec = Vec::with_capacity(ids.len());
                // TODO: there's surely a more ideomatic way...
                for (id, back) in ids.iter() {
                    ids_vec.push((*id, *back));
                }

                let (front_layer, back_layer) = match asset_name.as_ref() {
                    "hair" => (7,0),
                    "eyebrows" => (6,0),
                    "eyes" => (5,0),
                    "nose" => (4,0),
                    "mouth" => (3,0),
                    "face" => (2,0),
                    _ => (1,0),
                };

                let asset = Box::new(FileBackedAsset {
                    dir: entry.path(),
                    ids: ids_vec,
                    front_layer: front_layer,
                    back_layer: back_layer,
                });
                self.available_assets.insert(asset_name, asset);
            }
        }
        
        self.available_assets.insert("eyes".to_string(), Box::new(eyes::Eye {
            front_layer: 5,
        }));

        self.available_assets.insert("nose".to_string(), Box::new(placeable_feature::Feature {
            dir: Path::new("assets/nose").to_path_buf(),
            name: "nose".to_string(),
            ids: vec![(1, false)],
            front_layer: 4,
            back_layer: 0,
            is_symetric: false,
        }));

        self.available_assets.insert("mouth".to_string(), Box::new(placeable_feature::Feature {
            dir: Path::new("assets/mouth").to_path_buf(),
            name: "mouth".to_string(),
            ids: vec![(1, false)],
            front_layer: 3,
            back_layer: 0,
            is_symetric: false,
        }));
 
        self.available_assets.insert("ears".to_string(), Box::new(placeable_feature::Feature {
            dir: Path::new("assets/ears").to_path_buf(),
            name: "ears".to_string(),
            ids: vec![(1, false)],
            front_layer: 1,
            back_layer: 0,
            is_symetric: true,
        }));

        self.available_assets.insert("face".to_string(), Box::new(face::Face {
            front_layer: 2,
        }));
        Ok(())
    }

    pub fn generate(&self) -> Face {
        let mut rng = rand::thread_rng();
        let (_, skull) = &self.skulls.choose(&mut rng).unwrap();
        Face {
            face: self.available_assets["face"].choose(skull),
            ears: self.available_assets["ears"].choose(skull),
            eyes: self.available_assets["eyes"].choose(skull),
            eyebrows: self.available_assets["eyebrows"].choose(&skull),
            nose: self.available_assets["nose"].choose(&skull),
            mouth: self.available_assets["mouth"].choose(&skull),
            hair: self.available_assets["hair"].choose(&skull),
            skull: skull.clone(),
            pallete: self.select_pallete(),
        }

    }

    fn select_pallete(&self) -> Pallete {
        let mut rng = rand::thread_rng();

        let base_skin_color = (
            rng.gen_range(21.0, 35.0),
            163.0,
            rng.gen_range(50.0, 156.0),
        );

        let mut pallete = HashMap::new();

        let is_pale_complexion = base_skin_color.2 > 120.0;
        let base_skin_color = (base_skin_color.0 / 256.0, base_skin_color.1 / 256.0, base_skin_color.2 / 256.0);

        let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2 * 1.34);
        pallete.insert(
            "skin_color_0".to_string(),
            format!("#{:x}", rgb)
        );

        let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2 * 1.12);
        pallete.insert(
            "skin_color_1".to_string(),
            format!("#{:x}", rgb)
        );

        let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2);
        pallete.insert(
            "skin_color_2".to_string(),
            format!("#{:x}", rgb)
        );

        let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2 * 0.81);
        pallete.insert(
            "skin_color_3".to_string(),
            format!("#{:x}", rgb)
        );

        let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2 * 0.5);
        pallete.insert(
            "skin_color_4".to_string(),
            format!("#{:x}", rgb)
        );


        // TODO: Martin scale!
        let base_eye_color;
        match rng.gen_range(0, if is_pale_complexion { 4 } else { 2 }) {
            0 => { // Dark brown
                base_eye_color = (
                    rng.gen_range(18.0, 27.0),
                    222.0,
                    rng.gen_range(50.0, 82.0),
                );
            },
            2 => { // Hazel
                base_eye_color = (
                    rng.gen_range(18.0, 27.0),
                    222.0,
                    rng.gen_range(60.0, 92.0),
                );
            },
            3 => { // Blue
                base_eye_color = (
                    rng.gen_range(150.0, 160.0),
                    161.0,
                    rng.gen_range(104.0, 160.0),
                );
            },
            _ => { // Green
                base_eye_color = (
                    rng.gen_range(70.0, 90.0),
                    161.0,
                    rng.gen_range(104.0, 160.0),
                );
            },
        }

        pallete.insert("eye_color_1".to_string(), "#f2f2f2".to_string());
        let base_eye_color = (base_eye_color.0 / 256.0, base_eye_color.1 / 256.0, base_eye_color.2 / 256.0);
        let rgb = hslToRgb(base_eye_color.0, base_eye_color.1, base_eye_color.2);
        pallete.insert(
            "eye_color_2".to_string(),
            format!("#{:x}", rgb)
        );
        pallete.insert("eye_color_3".to_string(), "#00112b".to_string());

        let base_hair_color;
        match rng.gen_range(0, if is_pale_complexion { 4 } else { 1 }) {
            0 => { // Black Hair
                base_hair_color = (
                    rng.gen_range(18.0, 27.0),
                    222.0,
                    rng.gen_range(4.0, 20.0),
                );
            },
            1 => { // Brown Hair
                base_hair_color = (
                    rng.gen_range(18.0, 27.0),
                    222.0,
                    rng.gen_range(50.0, 82.0),
                );
            },
            2 => { // Red hair
                base_hair_color = (
                    rng.gen_range(6.0, 15.0),
                    222.0,
                    rng.gen_range(100.0, 140.0),
                );
            },
            _ => { // Blond hair
                base_hair_color = (
                    rng.gen_range(29.0, 40.0),
                    222.0,
                    rng.gen_range(100.0, 150.0),
                );
            },
        }

        let base_hair_color = (base_hair_color.0 / 256.0, base_hair_color.1 / 256.0, base_hair_color.2 / 256.0);
        let rgb = hslToRgb(base_hair_color.0, base_hair_color.1, base_hair_color.2 *
   1.2);
        pallete.insert(
            "hair_color_1".to_string(),
            format!("#{:x}", rgb)
        );

        let rgb = hslToRgb(base_hair_color.0, base_hair_color.1, base_hair_color.2);
        pallete.insert(
            "hair_color_2".to_string(),
            format!("#{:x}", rgb)
        );

        let rgb = hslToRgb(base_hair_color.0, base_hair_color.1, base_hair_color.2 * 0.8);
        pallete.insert(
            "hair_color_3".to_string(),
            format!("#{:x}", rgb)
        );

        pallete
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

#[derive(Clone)]
pub struct SVGFragment {
    contents: String,
    layer: u32,
}

impl SVGFragment {
    fn to_svg(&self, w: f64, x: f64, y: f64) -> String {
        let mut group = format!("<svg x='{}px' y='{}px' width='{}px' height='{}px' viewBox='0 0 210 210'>", x, y, w, w).to_string();

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
        let mut vertical_offset = 0.0;
        let mut horizontal_offset = 0.0;
        let mut in_this_row = 0;
        for frag in fragments {
            doc.push_str(&frag.to_svg(stride, horizontal_offset, vertical_offset));
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
    face: ConcreteAsset,
    ears: ConcreteAsset,
    eyes: ConcreteAsset,
    eyebrows: ConcreteAsset,
    nose: ConcreteAsset,
    mouth: ConcreteAsset,
    hair: ConcreteAsset,

    skull: Skull,
    pallete: Pallete,
}

impl Face {
    pub fn to_svg_fragment(&self) -> SVGFragment {
        let mut contents = String::new();

        let sources = vec![
            /*
            &self.hair,
            &self.eyebrows,
            */
            &self.ears,
            &self.nose,
            &self.face,
            &self.eyes,
            &self.mouth,
        ];

        let mut fragments = Vec::with_capacity(sources.len() + 1);

        for source in sources {
            fragments.extend(source.to_svg_fragments());
        }

        fragments.sort_unstable_by(|a, b| a.layer.cmp(&b.layer));

        for fragment in fragments {
            contents.push_str(&fragment.contents);
        }


        for (a, b) in &self.pallete {
            let pattern = format!("fill:{};", a);
            let replacement = format!("fill:{};", b);
            contents = contents.replace(&pattern, &replacement);

            let pattern = format!("stroke:{};", a);
            let replacement = format!("stroke:{};", b);
            contents = contents.replace(&pattern, &replacement);
        }

        SVGFragment {
            contents: contents,
            layer: 0,
        }
    }
}

type Pallete = HashMap<String, String>;


fn hslToRgb(h: f64, s: f64, l:f64) -> u32 {
    let r;
    let g;
    let b;

    if s == 0.0 {
        // achromatic
        r = l;
        g = l;
        b = l;
    } else {
        let hue2rgb = |p, q, mut t| {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.9;
            }
            if t < 1.0/6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0/2.0 {
                q
            } else if t < 2.0/3.0 {
                p + (q - p) * (2.0/3.0 - t) * 6.0
            } else {
                p
            }
        };

        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        r = hue2rgb(p, q, h + 1.0/3.0);
        g = hue2rgb(p, q, h);
        b = hue2rgb(p, q, h - 1.0/3.0);
    }

    let mut rgb = (r * 255.0) as u32;
    rgb = (rgb << 8) + (g * 255.0) as u32;
    rgb = (rgb << 8) + (b * 255.0) as u32;
    rgb
}

type SkullComponentCircle = ((f64, f64), f64);
type SkullComponentRect = (f64, f64, f64, f64);

#[derive(Clone, Deserialize)]
struct Skull {
    eyeball_left: SkullComponentCircle,
    eyeball_right: SkullComponentCircle,
    ear_right: SkullComponentRect,
    ear_left: SkullComponentRect,
    mouth: SkullComponentRect,
    nose: SkullComponentRect,

    outline: Vec<(f64, f64)>,
}
