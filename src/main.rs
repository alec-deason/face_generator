extern crate resvg;

extern crate face_generator;

use std::fs::File;
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

    let faces: Vec<Document> = (0..x_count * y_count).map(|_| generator.generate()).collect();

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
    let mut opt = resvg::Options::default();
    //let doc = Document::from_str(&String::from_utf8(output_data.clone()).unwrap()).unwrap();
    let doc2 = generator.generate();
    let rtree = resvg::usvg::Tree::from_dom(doc2, &opt.usvg);
    //eprintln!("{}", rtree.to_svgdom());

    //let rtree = resvg::usvg::Tree::from_str(&String::from_utf8(output_data.clone()).unwrap(), &opt.usvg).unwrap();

    let mut img = resvg::backend_cairo::render_to_image(&rtree, &opt).unwrap();
    let w = img.get_width() as u32;
    let h = img.get_height() as u32;
    let mut file = File::create("/tmp/output.png").unwrap();
    img.write_to_png(&mut file).unwrap();



    let mut file = File::create("/tmp/test.svg").unwrap();
    file.write_all(&output_data).unwrap();


    let mut opt = resvg::Options::default();
    opt.usvg.path = Some(Path::new("/tmp/test.svg").to_owned());

    let rtree = resvg::usvg::Tree::from_file(&"/tmp/test.svg", &opt.usvg).unwrap();
    let backend = resvg::default_backend();
    let img = backend.render_to_image(&rtree, &opt).unwrap();
    img.save(Path::new("/tmp/test.png"));
}
