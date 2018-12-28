use std::collections::HashMap;
use std::path::Path;

extern crate svgdom;
extern crate usvg;

use svgdom::{Document, Node, ElementId, AttributeId, FilterSvg, AttributeValue};
pub mod template;
pub mod feature;
pub mod complexion;

type Pallete = HashMap<String, String>;

#[derive(Copy, Clone)]
pub enum Guide {
    QuadGuide {
        ax: f64,
        ay: f64,
        bx: f64,
        by: f64,
        cx: f64,
        cy: f64,
        dx: f64,
        dy: f64,
    },
    CircleGuide { cx: f64, cy: f64, r: f64 },
}

impl Guide {
    fn new(node: &Node) -> Guide {
        match node.tag_id().unwrap() {
            ElementId::Path => {
                let attrs = node.attributes();
                if let Some(&AttributeValue::Path(ref path)) = attrs.get_value(AttributeId::D) {
                    if path.len() != 5 {
                        panic!();
                    }
                    let mut points = Vec::with_capacity(4);
                    for seg in path[..4].iter() {
                        let x = seg.x().unwrap();
                        let y = seg.y().unwrap();
                        points.push((x,y));
                    }
                    Guide::QuadGuide {
                        ax: points[0].0, ay: points[0].1,
                        bx: points[1].0, by: points[1].1,
                        cx: points[2].0, cy: points[2].1,
                        dx: points[3].0, dy: points[3].1,
                    }
                } else { panic!() }
            },
            ElementId::Rect => {
                let attrs = node.attributes();
                let x = match attrs.get_value(AttributeId::X).unwrap() {
                    AttributeValue::Length(x) => x.num,
                    _ => panic!(),
                };
                let y = match attrs.get_value(AttributeId::Y).unwrap() {
                    AttributeValue::Length(y) => y.num,
                    _ => panic!(),
                };
                let w = match attrs.get_value(AttributeId::Width).unwrap() {
                    AttributeValue::Length(w) => w.num,
                    _ => panic!(),
                };
                let h = match attrs.get_value(AttributeId::Height).unwrap() {
                    AttributeValue::Length(h) => h.num,
                    _ => panic!(),
                };
                let xx = x + w;
                let yy = y + h;
                Guide::QuadGuide {
                    ax: x, ay: y,
                    bx: xx, by: y,
                    cx: xx, cy: yy,
                    dx: x, dy: yy,
                }
            },
            ElementId::Circle => {
                let attrs = node.attributes();
                let cx = match attrs.get_value(AttributeId::Cx).unwrap() {
                    AttributeValue::Length(x) => x.num,
                    _ => panic!(),
                };
                let cy = match attrs.get_value(AttributeId::Cy).unwrap() {
                    AttributeValue::Length(y) => y.num,
                    _ => panic!(),
                };
                let r = match attrs.get_value(AttributeId::R).unwrap() {
                    AttributeValue::Length(r) => r.num,
                    _ => panic!(),
                };

                Guide::CircleGuide {
                    cx, cy, r,
                }
            },
            _ => panic!(),
        }
    }
}

struct Face {
    features: Vec<Node>,
}

pub struct Generator {
    templates: HashMap<String, template::Template>,
    features: HashMap<String, Vec<feature::Feature>>,
}

impl Generator {
    pub fn new(template_file: &Path, asset_files: &HashMap<String, &Path>) -> Generator {
        let templates = template::Template::all_from_file(template_file);
        let mut features = HashMap::with_capacity(asset_files.len());

        for (name, p) in asset_files {
            features.insert(name.to_owned(), feature::Feature::all_from_file(p));
        }

        Generator {templates, features}
    }

    pub fn generate(&mut self, name: &str) -> Document {
        let pallete = complexion::generate_pallete();
        self.templates[name].generate_from_features(&mut self.features, &pallete)
    }
}
