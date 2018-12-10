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
struct Guide(
    f64,
    f64,
    f64,
    #[serde(default)]
    Option<f64>,
);

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
        let guide:Guide = serde_json::from_str(&contents).unwrap();

        let asset_file = format!("{}{}.svg", id, suffix);
        let mut file = File::open(self.dir.join(asset_file)).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let sx;
        let sy;
        let gx;
        let gy;
        let xr;
        let yr;

        match self.name.as_ref() {
            "nose" => {
                let (x, y, w, h) = (guide.0, guide.1, guide.2, guide.3.unwrap());
                gx = x;
                gy = y;
                sx = skull.nose.0;
                sy = skull.nose.1;
                xr = skull.nose.2 / w;
                yr = skull.nose.3 / h;
            },
            "mouth" => {
                let (x, y, w, h) = (guide.0, guide.1, guide.2, guide.3.unwrap());
                gx = x;
                gy = y;
                sx = skull.mouth.0;
                sy = skull.mouth.1;
                xr = skull.mouth.2 / w;
                yr = skull.mouth.3 / h;
            },
            "hair" => {
                let (x, y, w, h) = (guide.0, guide.1, guide.2, guide.3.unwrap());
                gx = x;
                gy = y;
                sx = skull.hair.0;
                sy = skull.hair.1;
                xr = skull.hair.2 / w;
                yr = skull.hair.3 / h;
            },
            "ears" => {
                let (x, y, w, h) = (guide.0, guide.1, guide.2, guide.3.unwrap());
                gx = x;
                gy = y;
                match suffix.as_ref() {
                    "_left" => {
                        sx = skull.ear_left.0;
                        sy = skull.ear_left.1;
                        xr = skull.ear_left.2 / w;
                        yr = skull.ear_left.3 / h;
                    },
                    _ => {
                        sx = skull.ear_right.0;
                        sy = skull.ear_right.1;
                        xr = skull.ear_right.2 / w;
                        yr = skull.ear_right.3 / h;
                    },
                }
            },
            "eyes" => {
                if guide.3.is_some() {
                    panic!("Woops?");
                }
                let (x, y, r) = (guide.0, guide.1, guide.2);
                gx = x;
                gy = y;
                match suffix.as_ref() {
                    "_left" => {
                        sx = (skull.eyeball_left.0).0;
                        sy = (skull.eyeball_left.0).1;
                        xr = skull.eyeball_left.1 / r;
                        yr = skull.eyeball_left.1 / r;
                    },
                    _ => {
                        sx = (skull.eyeball_right.0).0;
                        sy = (skull.eyeball_right.0).1;
                        xr = skull.eyeball_right.1 / r;
                        yr = skull.eyeball_right.1 / r;
                    },
                }
            }
            _ => { panic!("Unknown asset"); },
        }

        let transform = format!("translate({}, {}) scale({}, {}) translate({}, {})", sx, sy, xr, yr, -gx,-gy);
        format!("<g transform='{}'>{}</g>", transform, contents)
    }
}
