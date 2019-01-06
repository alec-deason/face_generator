use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

use regex::Regex;

pub struct Weights {
    weights: Vec<(Regex, f32)>,
}

impl Weights {
    pub fn new(path: &Path) -> Weights {
        let mut weights = Vec::with_capacity(100);
        let file = File::open(path).unwrap();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            if !line.starts_with("#") & (line != "") {
                let prob_idx = line.rfind("|").expect("Pattern must have a probability");
                let re = Regex::new(&line[..prob_idx]).unwrap();
                let prob = line[prob_idx+1..].parse::<f32>().unwrap();
                weights.push((re, prob))
            }
        }
        Weights { weights }
    }

    pub fn for_path(&self, path: &str) -> f32 {
        for (re, prob) in &self.weights {
            if re.is_match(path) {
                return *prob
            }
        }
        1.0
    }
}