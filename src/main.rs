extern crate face_generator;

use std::path::Path;

fn main() {
    let generator = face_generator::FaceGenerator::new(Path::new("./assets"));
    let face = generator.generate();
    println!("{}", generator.to_svg(&face).unwrap());
}
