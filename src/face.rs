use super::{AbstractAssetTrait, ConcreteAssetTrait, ConcreteAsset, SVGFragment, Skull};


pub struct Face {
    pub front_layer: u32,
}

impl AbstractAssetTrait for Face {
    fn choose(&self, skull: &Skull) -> ConcreteAsset {

        Box::new((
            SVGFragment {
                contents: path(&skull.outline, "skin_color"),
                layer: self.front_layer,
            },
            None,
        ))
    }
}

fn path(points: &Vec<(f64, f64)>, css_class: &str) -> String {
    let mut s = format!(r#"<svg:path class="{}" style="fill-opacity:1;" d=""#, css_class);
    let mut first = true;
    for (x, y) in points {
        s.push_str(&format!("{} {},{} ", if first { "M" } else { "L" }, x, y));
        first = false;
    }
    s.push_str(r#"z"/>"#);
    s
}
