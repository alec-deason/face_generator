use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use regex::Regex;

pub struct Weights {
    weights: Vec<(Regex, Weight)>,
}

#[derive(Copy, Clone)]
pub enum Weight {
    Always,
    Sometimes(f32),
}

impl Weights {
    pub fn new(path: &Path) -> Weights {
        let mut weights = Vec::with_capacity(100);
        let file = File::open(path).unwrap();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            if !line.starts_with('#') & (line != "") {
                let prob_idx = line.rfind('|').expect("Pattern must have a probability");
                let re = Regex::new(&line[..prob_idx]).unwrap();
                let raw_prob = &line[prob_idx + 1..];
                let prob = if raw_prob == "always" {
                    Weight::Always
                } else {
                    Weight::Sometimes(raw_prob.parse::<f32>().unwrap())
                };
                weights.push((re, prob))
            }
        }
        Weights { weights }
    }

    pub fn for_path(&self, path: &str) -> Weight {
        for (re, prob) in &self.weights {
            if re.is_match(path) {
                return *prob;
            }
        }
        Weight::Sometimes(1.0)
    }
}
