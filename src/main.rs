extern crate face_generator;

use std::path::Path;

fn main() {
    let generator = face_generator::Generator::new(
        Path::new("assets/skulls.svg"),
        &vec![
            ("nose".to_owned(), Path::new("assets/nose.svg")),
        ].iter().cloned().collect()
    );
    /*
    let generator = face_generator::FaceGenerator::new(Path::new("./assets"));
    let faces:Vec<face_generator::Face> = (0..width*width).map(|_| generator.generate()).collect();

    let fragments:Vec<face_generator::SVGFragment> = faces.iter().map(|face| {
        face.to_svg_fragment()
    }).collect();

    println!("{}", face_generator::svg_grid(&fragments, width));
    */
}
