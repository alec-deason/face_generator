extern crate resvg;

extern crate face_generator;

use std::fs::File;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use resvg::svgdom::{Attribute, AttributeId, AttributeValue, Document, ElementId, Transform, ViewBox, WriteBuffer};

fn main() {
    let _resvg = resvg::init();
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
        AttributeId::ViewBox,
        AttributeValue::ViewBox(ViewBox::new(
            0.0,
            0.0,
            total_width,
            total_height,
        )),
    ));
    svg.set_attribute(Attribute::new(AttributeId::X, AttributeValue::Number(0.0)));
    svg.set_attribute(Attribute::new(AttributeId::Y, AttributeValue::Number(0.0)));
    svg.set_attribute(Attribute::new(AttributeId::Width, AttributeValue::Number(total_width)));
    svg.set_attribute(Attribute::new(AttributeId::Height, AttributeValue::Number(total_height)));

    let faces: Vec<Document> = (0..x_count * y_count).map(|_| generator.generate(&HashMap::new())).collect();

    for x in 0..x_count {
        for y in 0..y_count {
            let face = &faces[x + y * x_count];
            face.svg_element().unwrap().set_attribute(Attribute::new(
                AttributeId::ViewBox,
                AttributeValue::ViewBox(ViewBox::new(
                    0.0,
                    0.0,
                    210.0,
                    210.0,
                )),
            ));
            face.svg_element().unwrap().set_attribute(Attribute::new(AttributeId::X, AttributeValue::Number(x as f64 * width)));
            face.svg_element().unwrap().set_attribute(Attribute::new(AttributeId::Y, AttributeValue::Number(y as f64 * height)));
            face.svg_element().unwrap().set_attribute(Attribute::new(AttributeId::Width, AttributeValue::Number(width)));
            face.svg_element().unwrap().set_attribute(Attribute::new(AttributeId::Height, AttributeValue::Number(height)));
            svg.append(face.root());
        }
    }

    doc.root().append(svg);
    let mut output_data = Vec::new();
    doc.write_buf(&mut output_data);
    io::stdout().write_all(&output_data).unwrap();
}
