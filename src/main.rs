extern crate svgdom;

extern crate face_generator;

use std::io::{self, Write};
use std::path::Path;
use svgdom::{Attribute, AttributeValue, Document, ElementId, Transform, ViewBox, WriteBuffer};

fn main() {
    let mut generator = face_generator::Generator::new(&Path::new("assets"));

    let width = 6;
    let height = 3;

    let mut doc = Document::new();
    let mut svg = doc.create_element(ElementId::Svg);
    svg.set_attribute(Attribute::new(
        "viewbox",
        AttributeValue::ViewBox(ViewBox::new(
            0.0,
            0.0,
            (width as f64) * 210.0,
            (height as f64) * 210.0,
        )),
    ));

    let faces: Vec<Document> = (0..width * height).map(|_| generator.generate()).collect();

    for x in 0..width {
        for y in 0..height {
            let face = &faces[x + y * width];
            let transform = Transform::new_translate(x as f64 * 210.0, y as f64 * 210.0);
            face.svg_element().unwrap().set_attribute(Attribute::new(
                "transform",
                AttributeValue::Transform(transform),
            ));
            svg.append(face.root());
        }
    }

    doc.root().append(svg);

    let mut output_data = Vec::new();
    doc.write_buf(&mut output_data);
    io::stdout().write_all(&output_data).unwrap();
}
