use std::path::{PathBuf};
use rand::seq::SliceRandom;
use std::fs::{File};
use std::io::prelude::*;

use serde_json;

use super::{AbstractAssetTrait, ConcreteAsset, SVGFragment, Skull};


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
    #[serde(default)]
    Option<f64>,
    #[serde(default)]
    Option<f64>,
    #[serde(default)]
    Option<f64>,
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
        let mut use_css = true;

        let transform = match self.name.as_ref() {
            "nose" => {
                transformation_from_quad(skull.nose, (guide.0, guide.1, guide.2, guide.3.unwrap(), guide.4.unwrap(), guide.5.unwrap(), guide.6.unwrap(), guide.7.unwrap()))
            },
            "mouth" => {
                transformation_from_quad(skull.mouth, (guide.0, guide.1, guide.2, guide.3.unwrap(), guide.4.unwrap(), guide.5.unwrap(), guide.6.unwrap(), guide.7.unwrap()))
            },
            "hair" => {
                transformation_from_quad(skull.hair, (guide.0, guide.1, guide.2, guide.3.unwrap(), guide.4.unwrap(), guide.5.unwrap(), guide.6.unwrap(), guide.7.unwrap()))
            },
            "ears" => {
                match suffix.as_ref() {
                    "_left" => {
                        transformation_from_quad(skull.ear_left, (guide.0, guide.1, guide.2, guide.3.unwrap(), guide.4.unwrap(), guide.5.unwrap(), guide.6.unwrap(), guide.7.unwrap()))
                    },
                    _ => {
                        transformation_from_quad(skull.ear_right, (guide.0, guide.1, guide.2, guide.3.unwrap(), guide.4.unwrap(), guide.5.unwrap(), guide.6.unwrap(), guide.7.unwrap()))
                    },
                }
            },
            "eyes" => {
                if guide.3.is_some() {
                    panic!("Woops?");
                }
                use_css = false;
                match suffix.as_ref() {
                    "_left" => {
                        transformation_from_circle(skull.eyeball_left, (guide.0, guide.1, guide.2))
                    },
                    _ => {
                        transformation_from_circle(skull.eyeball_right, (guide.0, guide.1, guide.2))
                    },
                }
            }
            _ => { panic!("Unknown asset"); },
        };

        if use_css {
            format!("<g style='transform:{}'>{}</g>", transform, contents)
        } else {
            format!("<g transform='{}'>{}</g>", transform, contents)
        }
    }
}

fn transformation_from_quad(feat: (f64, f64, f64, f64, f64, f64, f64, f64), guide: (f64, f64, f64, f64, f64, f64, f64, f64)) -> String {
    let m = transform2d(
        guide.0, guide.1, guide.2, guide.3, guide.4, guide.5, guide.6, guide.7,
        feat.0, feat.1, feat.2, feat.3, feat.4, feat.5, feat.6, feat.7,
    );
    format!("matrix3d({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})", m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], m[10], m[11], m[12], m[13], m[14], m[15]).to_string()
}

fn transformation_from_circle(feature: ((f64, f64), f64), guide: (f64, f64, f64)) -> String {
    let (gx, gy, gr) = guide;
    let (fx, fy, fr) = ((feature.0).0, (feature.0).1, feature.1);
    let xr = fr / gr;
    let yr = fr / gr;
    format!("translate({}, {}) scale({}, {}) translate({}, {})", fx, fy, xr, yr, -gx,-gy)
}

fn adj(m: [f64; 9]) -> [f64; 9] { // Compute the adjugate of m
  [
    m[4]*m[8]-m[5]*m[7], m[2]*m[7]-m[1]*m[8], m[1]*m[5]-m[2]*m[4],
    m[5]*m[6]-m[3]*m[8], m[0]*m[8]-m[2]*m[6], m[2]*m[3]-m[0]*m[5],
    m[3]*m[7]-m[4]*m[6], m[1]*m[6]-m[0]*m[7], m[0]*m[4]-m[1]*m[3]
  ]
}

fn multmm(a: [f64; 9], b: [f64; 9]) -> [f64; 9] { // multiply two matrices
  let mut c = [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0];
  for i in 0..3 {
    for j in 0..3 {
      let mut cij = 0.0;
      for k in 0..3 {
        cij += a[3*i + k]*b[3*k + j];
      }
      c[3*i + j] = cij;
    }
  }
  c
}

fn multmv(m: [f64; 9], v: [f64; 3]) -> [f64; 3] { // multiply matrix and vector
  [
    m[0]*v[0] + m[1]*v[1] + m[2]*v[2],
    m[3]*v[0] + m[4]*v[1] + m[5]*v[2],
    m[6]*v[0] + m[7]*v[1] + m[8]*v[2]
  ]
}

fn basisToPoints(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) -> [f64; 9] {
  let m = [
    x1, x2, x3,
    y1, y2, y3,
     1.0,  1.0,  1.0
  ];
  let v = multmv(adj(m), [x4, y4, 1.0]);
  multmm(m, [
    v[0], 0.0, 0.0,
    0.0, v[1], 0.0,
    0.0, 0.0, v[2]
  ])
}

fn general2DProjection(
  x1s: f64, y1s: f64, x1d: f64, y1d:f64,
  x2s: f64, y2s: f64, x2d: f64, y2d:f64,
  x3s: f64, y3s: f64, x3d: f64, y3d:f64,
  x4s: f64, y4s: f64, x4d: f64, y4d:f64
) -> [f64; 9]{
  let s = basisToPoints(x1s, y1s, x2s, y2s, x3s, y3s, x4s, y4s);
  let d = basisToPoints(x1d, y1d, x2d, y2d, x3d, y3d, x4d, y4d);
  multmm(d, adj(s))
}

fn transform2d(
    x1s: f64, y1s: f64, x2s: f64, y2s: f64, x3s: f64, y3s: f64, x4s: f64, y4s: f64,
    x1d: f64, y1d: f64, x2d: f64, y2d: f64, x3d: f64, y3d: f64, x4d: f64, y4d: f64,
) -> [f64; 16] {
  let mut t = general2DProjection(x1s, y1s, x1d, y1d, x2s, y2s, x2d, y2d, x3s, y3s, x3d, y3d, x4s, y4s, x4d, y4d);
  for i in 0..9 {
      t[i] = t[i]/t[8];
  }
  let t = [t[0], t[3], 0.0, t[6],
       t[1], t[4], 0.0, t[7],
       0.0   , 0.0   , 1.0, 0.0   ,
       t[2], t[5], 0.0, t[8]];
  t
}
