extern crate svgdom;

extern crate face_generator;

use std::io::{self, Write};
use std::path::Path;
use svgdom::{Attribute, AttributeValue, Document, ElementId, Transform, ViewBox, WriteBuffer};

fn main() {
    let mut generator = face_generator::Generator::new(&Path::new("assets"));

    let total_width = 6.0 * 210.0;
    let total_height = 3.0 * 210.0;
    let x_count = 6;
    let y_count = 3;
    let width = total_width / x_count as f64;
    let height = total_height / y_count as f64;

    let mut doc = Document::new();
    let mut svg = doc.create_element(ElementId::Svg);
    svg.set_attribute(Attribute::new(
        "viewbox",
        AttributeValue::ViewBox(ViewBox::new(
            0.0,
            0.0,
            total_width,
            total_height,
        )),
    ));
    svg.set_attribute(Attribute::new("x", AttributeValue::Number(0.0)));
    svg.set_attribute(Attribute::new("y", AttributeValue::Number(0.0)));
    svg.set_attribute(Attribute::new("width", AttributeValue::Number(total_width)));
    svg.set_attribute(Attribute::new("height", AttributeValue::Number(total_height)));

    let faces: Vec<Document> = (0..x_count * y_count).map(|_| generator.generate()).collect();

    for x in 0..x_count {
        for y in 0..y_count {
            let face = &faces[x + y * x_count];
            /*
            let transform = Transform::new_translate(x as f64 * 210.0, y as f64 * 210.0);
            face.svg_element().unwrap().set_attribute(Attribute::new(
                "transform",
                AttributeValue::Transform(transform),
            ));
            */
            face.svg_element().unwrap().set_attribute(Attribute::new(
                "viewbox",
                AttributeValue::ViewBox(ViewBox::new(
                    0.0,
                    0.0,
                    width,
                    height,
                )),
            ));
            face.svg_element().unwrap().set_attribute(Attribute::new("x", AttributeValue::Number(x as f64 * 210.0)));
            face.svg_element().unwrap().set_attribute(Attribute::new("y", AttributeValue::Number(y as f64 * 210.0)));
            face.svg_element().unwrap().set_attribute(Attribute::new("width", AttributeValue::Number(width)));
            face.svg_element().unwrap().set_attribute(Attribute::new("height", AttributeValue::Number(height)));
            svg.append(face.root());
        }
    }

    doc.root().append(svg);

    let mut output_data = Vec::new();
    doc.write_buf(&mut output_data);
    io::stdout().write_all(&output_data).unwrap();
}
