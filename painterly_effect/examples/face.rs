extern crate face_generator;
extern crate resvg;
extern crate painterly_effect;

use std::collections::HashMap;
use std::path::Path;
use resvg::backend_cairo::render_to_image;
use resvg::svgdom::{Attribute, AttributeId, AttributeValue, Document};
use resvg::usvg;
use image as im;

fn main() {
    let mut generator = face_generator::Generator::new(Path::new("/home/alec/Code/face_generator/assets"));
    let mut attributes = HashMap::new();
    attributes.insert("species".to_string(), ["human".to_string()].iter().cloned().collect());
    attributes.insert("age".to_string(), ["adult".to_string()].iter().cloned().collect());
    let (mut doc, _) = generator.generate(&attributes);
    let width = 512.0;
    let height = 512.0;

    let mut svg = doc.svg_element().unwrap();
    svg.set_attribute(Attribute::new(
        AttributeId::Width,
        AttributeValue::Number(width),
    ));
    svg.set_attribute(Attribute::new(
        AttributeId::Height,
        AttributeValue::Number(width),
    ));

    let rtree = usvg::Tree::from_dom(doc, &usvg::Options::default()).unwrap();

    let mut img = render_to_image(
        &rtree,
        &resvg::Options {
            fit_to: resvg::FitTo::Width(width as u32),
            ..resvg::Options::default()
        },
    ).unwrap();
    let w = img.get_width() as u32;
    let h = img.get_height() as u32;
    let data: Vec<u8> = img.get_data().unwrap().iter().cloned().collect();
     let mut new_data = Vec::with_capacity(data.len());
    for i in 0..data.len() / 4 {
        new_data.push(data[i * 4 + 2]);
        new_data.push(data[i * 4 + 1]);
        new_data.push(data[i * 4 + 0]);
        new_data.push(255);
    }


    let img: im::ImageBuffer<im::Rgba<u8>, Vec<u8>> = im::ImageBuffer::from_raw(w, h, new_data).unwrap();
    //let img: im::ImageBuffer<im::Rgba<u8>, Vec<u8>> = im::open(&Path::new(&"/tmp/input.jpg")).unwrap().to_rgba();

    painterly_effect::apply_brush_agents_effect(&img).save("/tmp/test.png").unwrap();
}
