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
        }
    }
    pub fn to_svg(&self, face: &Face) -> Result<String, std::io::Error> {
        let mut doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>

<svg
xmlns:svg="http://www.w3.org/2000/svg"
xmlns="http://www.w3.org/2000/svg"
width="210mm"
height="297mm"
viewBox="0 0 210 297"
version="1.1"
>
        "#.to_owned();

        doc.push_str(&self.asset_to_string("ears", face.ears)?);
        doc.push_str(&self.asset_to_string("face", face.face)?);
        doc.push_str(&self.asset_to_string("eyes", face.eyes)?);
        doc.push_str(&self.asset_to_string("mouth", face.mouth)?);
        doc.push_str(&self.asset_to_string("nose", face.nose)?);
        doc.push_str(&self.asset_to_string("hair", face.hair)?);

        doc.push_str("</svg>");
        Ok(doc)
    }

    fn asset_to_string(&self, asset: &str, id: u32) -> Result<String, std::io::Error> {
        let asset_file = format!("{}.svg", id);
        let mut file = File::open(self.assets_dir.join(asset).join(asset_file))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        Ok(contents)
    }
}

pub struct Face {
    face: u32,
    ears: u32,
    eyes: u32,
    nose: u32,
    mouth: u32,
    hair: u32,
}
