use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::collections::hash_map::HashMap;

use svgdom::{Document, Node, FilterSvg, ElementId, WriteBuffer};
use usvg;

use super::{Guide, Pallete};
use super::feature::Feature;

pub struct Template {
    guides: Vec<(String, Guide)>,
}

impl Template {
    pub fn new(tree: &Node) -> Template {
        let mut guides = Vec::new();
        for (_, node) in tree.descendants().svg() {
            if node.has_id() {
                let id = node.id();
                if id.starts_with("guide_") {
                    let end = id.find('-').unwrap_or_else(|| id.len());
                    let feature_name = &id[6..end];
                    let guide = Guide::new(&node);
                    guides.push((feature_name.to_owned(), guide));
                }
            }
        }
        Template {
            guides,
        }
    }

    pub fn all_from_file(path: &Path) -> HashMap<String, Template> {
        let mut templates = HashMap::new();

        let doc = usvg::Tree::from_file(path, &usvg::Options { keep_named_groups: true, .. usvg::Options::default() });
        let doc = doc.unwrap().to_svgdom();
        let mut output_data = Vec::new();
        doc.write_buf(&mut output_data);

        let mut f = File::create("/tmp/skulls.svg").unwrap();
f.write_all(&output_data).unwrap();
        for node in doc.root().descendants() {
            if node.has_id() {
                let id = node.id();
                if id.starts_with("skull_") {
                    println!("found template: {}", id);
                    let template_name = &id[6..];
                    let template = Self::new(&node);
                    templates.insert(template_name.to_owned(), template);
                }
            }
        }
        templates
    }

    pub fn generate_from_features(&self, features: &mut HashMap<String, Vec<Feature>>, pallete: &Pallete) -> Document {
        let mut doc = Document::new();
        let mut svg = doc.create_element(ElementId::Svg);

        for (name, guide) in &self.guides {
            match features.get_mut(name) {
                Some(feature) => {
                    let node = feature[0].aligned_contents(guide, pallete);
                    svg.append(node);
                },
                None => (),
            }
        }
        doc.root().append(svg);

        doc
    }
}
