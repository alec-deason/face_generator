use std::path::Path;
use std::io::Read;
use std::fs::File;

use svgdom::{Document, Node, ElementId, ParseOptions, FilterSvg, AttributeValue};

use super::Guide;

pub struct Feature {
    guide: Guide,
    name: String,
    contents: Node,
}

impl Feature {
    pub fn all_from_file(path: &Path) -> Vec<Feature> {
        let mut features_svg = Vec::new();
        let mut guide = None;

        let mut file = File::open(path).unwrap();
        let length = file.metadata().unwrap().len() as usize;

        let mut input_data = String::with_capacity(length + 1);
        file.read_to_string(&mut input_data).unwrap();

        let opt = ParseOptions {
            skip_invalid_css: true,
            .. ParseOptions::default()
        };

        let doc = Document::from_str_with_opt(&input_data, &opt).unwrap();
        for (_, node) in doc.root().descendants().svg() {
            if node.has_id() {
                let id = node.id();
                if *id == "guide" {
                    guide = Some(Guide::new(&node.children().next().unwrap()));
                } else if id.starts_with("feature_") {
                    features_svg.push(node.clone());
                }
            }
        }
        let guide = guide.unwrap();
        features_svg.iter().map(|f| {
            Feature {
                guide: guide,
                name: "test".to_owned(),
                contents: f.clone(),
            }
        }).collect()
    }
}
