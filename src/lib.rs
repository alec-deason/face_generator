use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

extern crate rand;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate resvg;
#[macro_use]
extern crate serde_derive;
extern crate palette;

use rand::prelude::*;
use rand::Rng;

use resvg::svgdom::{AttributeId, AttributeValue, Document, ElementId, Node};

pub mod color_scheme;
pub mod template;
pub mod weights;

type Palette = HashMap<String, String>;

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
    CircleGuide {
        cx: f64,
        cy: f64,
        r: f64,
    },
}

impl Guide {
    fn new(node: &Node) -> Self {
        match node.tag_id().unwrap() {
            ElementId::Path => {
                let attrs = node.attributes();
                if let Some(&AttributeValue::Path(ref path)) = attrs.get_value(AttributeId::D) {
                    match path.len() {
                        5 => {
                            // This is a quadrilateral
                            let mut points = Vec::with_capacity(4);
                            for seg in path[..4].iter() {
                                let x = seg.x().unwrap();
                                let y = seg.y().unwrap();
                                points.push((x, y));
                            }
                            Guide::QuadGuide {
                                ax: points[0].0,
                                ay: points[0].1,
                                bx: points[1].0,
                                by: points[1].1,
                                cx: points[2].0,
                                cy: points[2].1,
                                dx: points[3].0,
                                dy: points[3].1,
                            }
                        }
                        4 => {
                            // This is a triangle, which we treat as a translate-and-scale
                            let mut points = Vec::with_capacity(3);
                            for seg in path[..3].iter() {
                                let x = seg.x().unwrap();
                                let y = seg.y().unwrap();
                                points.push((x, y));
                            }

                            let cx: f64 = points.iter().map(|p| p.0).sum::<f64>() / 3.0;
                            let cy: f64 = points.iter().map(|p| p.1).sum::<f64>() / 3.0;
                            let r: f64 = ((points[0].0 - cx).powf(2.0)
                                + (points[0].1 - cy).powf(2.0))
                            .sqrt();
                            Guide::CircleGuide { cx, cy, r }
                        }
                        _ => panic!(),
                    }
                } else {
                    panic!()
                }
            }
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
                    ax: x,
                    ay: y,
                    bx: xx,
                    by: y,
                    cx: xx,
                    cy: yy,
                    dx: x,
                    dy: yy,
                }
            }
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

                Guide::CircleGuide { cx, cy, r }
            }
            _ => panic!(),
        }
    }
}

pub struct GenerationContext<'a> {
    templates: &'a HashMap<String, HashMap<String, template::Template>>,
    palette: &'a Palette,
    weights: &'a weights::Weights,
    seeds: RefCell<HashMap<(String, String), u64>>,
}

impl<'a> GenerationContext<'a> {
    pub fn new(
        templates: &'a HashMap<String, HashMap<String, template::Template>>,
        palette: &'a Palette,
        weights: &'a weights::Weights,
    ) -> GenerationContext<'a> {
        GenerationContext {
            templates,
            palette,
            weights,
            seeds: RefCell::new(HashMap::new()),
        }
    }

    pub fn use_optional(&self, path: &str, name: &str) -> bool {
        let full_path = format!("{}:option:{}", path, name);
        let mut base_rng = rand::thread_rng();
        let seed: u64 = *self
            .seeds
            .borrow_mut()
            .entry((name.to_owned(), "".to_string()))
            .or_insert_with(|| base_rng.gen());
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
        let weight = self.weights.for_path(&full_path);
        match weight {
            weights::Weight::Always => true,
            weights::Weight::Sometimes(prob) => rng.gen::<f32>() < prob,
        }
    }

    pub fn choose_template(&self, path: &str, name: &str, name_variant: &str) -> Option<(&template::Template, String)> {
        let is_back;
        let name = if name.ends_with("_back") {
            is_back = true;
            &name[..name.len() - 5]
        } else {
            is_back = false;
            name
        };

        let full_path = format!("{}:{}", path, name);
        let mut base_rng = rand::thread_rng();
        let seed: u64 = *self
            .seeds
            .borrow_mut()
            .entry((name.to_owned(), name_variant.to_owned()))
            .or_insert_with(|| base_rng.gen());
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
        let prob = self.weights.for_path(&full_path);
        let do_choose = match prob {
            weights::Weight::Always => true,
            weights::Weight::Sometimes(prob) => rng.gen::<f32>() < prob,
        };
        if do_choose {
            if let Some(variations) = &self.templates.get(name) {
                let weights: Vec<((&String, &template::Template), weights::Weight)> = variations
                    .iter()
                    .map(|v| (v, self.weights.for_path(&format!("{}:{}", full_path, v.0))))
                    .collect();
                let variation;
                if let Some((choice, _)) = weights.iter().find(|(_, w)| match w { weights::Weight::Always => true, weights::Weight::Sometimes(_) => false}) {
                    variation = choice;
                } else {
                    let total_weight:f32 = weights.iter().map(|(_, w)| match w { weights::Weight::Always => panic!(), weights::Weight::Sometimes(w) => w }).sum();
                    let (choice, _) = weights.choose_weighted(&mut rng, |e| match e.1 { weights::Weight::Always => panic!(), weights::Weight::Sometimes(w) => w } /total_weight).unwrap();
                    variation = choice;
                }
                if is_back {
                    if self.templates.contains_key(&format!("{}_back", name)) {
                        let variations = &self.templates[&format!("{}_back", name)];
                        if variations.contains_key(variation.0) {
                            Some((
                                &variations[variation.0],
                                format!("{}_back:{}", full_path, variation.0),
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    Some((
                        &variations[variation.0],
                        format!("{}:{}", full_path, variation.0),
                    ))
                }
            } else {
                eprintln!("No templates for '{}'", name);
                None
            }
        } else {
            None
        }
    }
}

pub struct Generator {
    templates: HashMap<String, HashMap<String, template::Template>>,
    asset_dir: PathBuf,
    weights: weights::Weights,
}

impl Generator {
    pub fn new(asset_dir: &Path) -> Self {
        let mut templates = HashMap::with_capacity(20);

        for entry in asset_dir.read_dir().unwrap() {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let name = entry.file_name().to_owned().into_string().unwrap();
                        templates.insert(name, template::Template::from_directory(&entry.path()));
                    }
                }
            }
        }

        let weights = weights::Weights::new(&asset_dir.join("probabilities"));
        let asset_dir = asset_dir.to_owned();

        Self { templates, asset_dir, weights }
    }

    pub fn generate(&mut self) -> Document {
        let mut rng = rand::thread_rng();
        let (species, _) = [("human", 0.6), ("dwarf", 0.3), ("elf", 0.3), ("goblin", 0.02), ("cyclops", 0.02)].choose_weighted(&mut rng, |s| s.1).unwrap();
        let (palette_path, palette) = &color_scheme::palette_from_file(&self.asset_dir.join("palette.json"), species);
        let context = GenerationContext::new(&self.templates, &palette, &self.weights);
        let sex = ["male", "female"].choose(&mut rng).unwrap();
        let (frame, full_path) = context
            .choose_template(&format!("{}:{}", palette_path, sex), "frame", "")
            .unwrap();
        frame.generate_from_context(&context, &full_path)
    }
}
