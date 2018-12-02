extern crate face_generator;

use std::path::Path;

fn main() {
    let width = 10;
    let generator = face_generator::FaceGenerator::new(Path::new("./assets"));
    let faces:Vec<face_generator::Face> = (0..width*width).map(|_| generator.generate()).collect();

    let fragments:Vec<face_generator::SVGFragment> = faces.iter().map(|face| {
        generator.to_svg_fragment(&face).unwrap()
    }).collect();

    println!("{}", face_generator::svg_grid(&fragments, width));
}
