extern crate svgdom;

extern crate face_generator;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use svgdom::WriteBuffer;

fn main() {
    let mut generator = face_generator::Generator::new(
        Path::new("assets/skulls.svg"),
        &vec![
            ("hair".to_owned(), Path::new("assets/hair.svg")),
            ("nose".to_owned(), Path::new("assets/nose.svg")),
            ("mouth".to_owned(), Path::new("assets/mouth.svg")),
            ("eye".to_owned(), Path::new("assets/eye.svg")),
            ("mandible".to_owned(), Path::new("assets/mandible.svg")),
            (
                "cheek_bones".to_owned(),
                Path::new("assets/cheek_bones.svg"),
            ),
            ("skull_cap".to_owned(), Path::new("assets/skull_cap.svg")),
            ("ear".to_owned(), Path::new("assets/ear.svg")),
        ]
        .iter()
        .cloned()
        .collect(),
    );

    let args: Vec<_> = env::args().collect();
    let face = generator.generate(&args[1]);

    let mut output_data = Vec::new();
    face.write_buf(&mut output_data);

    let mut f = File::create("/tmp/test.svg").unwrap();
    f.write_all(&output_data).unwrap();
}
