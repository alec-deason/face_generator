use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::collections::hash_map::HashMap;

use svgdom::{Document, Node, ElementId, AttributeId, FilterSvg, AttributeValue};

use super::Guide;

pub struct Template {
    guides: Vec<(String, Guide)>,
}

impl Template {
    pub fn new(tree: &Node) -> Template {
        let mut guides = Vec::new();
        for (_, node) in tree.descendants().svg() {
            if node.has_id() {
                let id = node.id();
                println!("{}", id);
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

        let mut file = File::open(path).unwrap();
        let length = file.metadata().unwrap().len() as usize;

        let mut input_data = String::with_capacity(length + 1);
        file.read_to_string(&mut input_data).unwrap();

        let doc = Document::from_str(&input_data).unwrap();
        for (_, node) in doc.root().descendants().svg() {
            if node.has_id() {
                let id = node.id();
                if id.starts_with("skull_") {
                    let template_name = &id[6..];
                    let template = Self::new(&node);
                    println!("{}", template.guides.len());
                    templates.insert(template_name.to_owned(), template);
                }
            }
        }
        templates
    }
}
