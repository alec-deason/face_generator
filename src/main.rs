extern crate face_generator;

use std::io::Write;
use std::fs::File;

use std::path::Path;
use svgdom::WriteBuffer;

fn main() {
    let mut generator = face_generator::Generator::new(
        Path::new("assets/skulls.svg"),
        &vec![
            ("nose".to_owned(), Path::new("assets/nose.svg")),
            ("mouth".to_owned(), Path::new("assets/mouth.svg")),
            ("hair".to_owned(), Path::new("assets/hair.svg")),
            ("eyeball_right".to_owned(), Path::new("assets/eye_right.svg")),
            ("eyeball_left".to_owned(), Path::new("assets/eye_left.svg")),
        ].iter().cloned().collect()
    );

    let face = generator.generate();

    let mut output_data = Vec::new();
    face.write_buf(&mut output_data);

    let mut f = File::create("/tmp/test.svg").unwrap();
    f.write_all(&output_data).unwrap();
    /*
    let generator = face_generator::FaceGenerator::new(Path::new("./assets"));
    let faces:Vec<face_generator::Face> = (0..width*width).map(|_| generator.generate()).collect();

    let fragments:Vec<face_generator::SVGFragment> = faces.iter().map(|face| {
        face.to_svg_fragment()
    }).collect();

    println!("{}", face_generator::svg_grid(&fragments, width));
    */
}
