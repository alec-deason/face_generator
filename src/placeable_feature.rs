use std::path::{PathBuf};
use rand::seq::SliceRandom;
use std::fs::{self, File};
use std::io::prelude::*;

use svgtypes::Path;
use serde_json;

use super::{AbstractAssetTrait, ConcreteAssetTrait, ConcreteAsset, SVGFragment, Skull};


pub struct Feature {
    pub dir: PathBuf,
    pub name: String,
    pub ids: Vec<(u32, bool)>,
    pub front_layer: u32,
    pub back_layer: u32,
    pub is_symetric: bool,
}

#[derive(Copy, Clone, Deserialize)]
struct Guide(f64, f64, f64, f64);

impl AbstractAssetTrait for Feature {
    fn choose(&self, skull: &Skull) -> ConcreteAsset {
        let mut rng = rand::thread_rng();
        let (id, has_back) = self.ids.choose(&mut rng).unwrap();

        let contents = if self.is_symetric {
            let mut s = self._construct(*id, false, "_left", skull);
            s.push_str(&self._construct(*id, false, "_right", skull));
            s
        } else {
            self._construct(*id, false, "", skull)
        };

        Box::new((
            SVGFragment {
                contents: contents,
                layer: self.front_layer,
            },
            if *has_back {
                let contents = if self.is_symetric {
                    let mut s = self._construct(*id, true, "_left", skull);
                    s.push_str(&self._construct(*id, true, "_right", skull));
                    s
                } else {
                    self._construct(*id, true, "", skull)
                };
                Some(
                    SVGFragment {
                        contents: contents,
                        layer: self.back_layer,
                    }
                )
            } else { None }
        ))
    }
}

impl Feature {
    fn _construct(&self, id: u32, back: bool, suffix: &str, skull: &Skull) -> String {
        let mut suffix = suffix.to_string();
        if back {
            suffix.push_str("_back");
        }

        let mut file = File::open(self.dir.join(format!("{}{}.json", id, suffix))).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let guide: Guide = serde_json::from_str(&contents).unwrap();

        let asset_file = format!("{}{}.svg", id, suffix);
        let mut file = File::open(self.dir.join(asset_file)).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let sx;
        let sy;
        let xr;
        let yr;

        match self.name.as_ref() {
            "nose" => {
                sx = skull.nose.0;
                sy = skull.nose.1;
                xr = skull.nose.2 / guide.2;
                yr = skull.nose.3 / guide.3;
            },
            "mouth" => {
                sx = skull.mouth.0;
                sy = skull.mouth.1;
                xr = skull.mouth.2 / guide.2;
                yr = skull.mouth.3 / guide.3;
            },
            "ears" => {
                match suffix.as_ref() {
                    "_left" => {
                        sx = skull.ear_left.0;
                        sy = skull.ear_left.1;
                        xr = skull.ear_left.2 / guide.2;
                        yr = skull.ear_left.3 / guide.3;
                    },
                    _ => {
                        sx = skull.ear_right.0;
                        sy = skull.ear_right.1;
                        xr = skull.ear_right.2 / guide.2;
                        yr = skull.ear_right.3 / guide.3;
                    },
                }
            },
            _ => { panic!("Unknown asset"); },
        }

        let transform = format!("translate({}, {}) scale({}, {}) translate({}, {})", sx, sy, xr, yr, -guide.0,-guide.1);
        format!("<g transform='{}'>{}</g>", transform, contents)
    }
}
