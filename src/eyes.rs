use super::{AbstractAssetTrait, ConcreteAssetTrait, ConcreteAsset, SVGFragment, Skull};


pub struct Eye {
    pub front_layer: u32,
}

impl AbstractAssetTrait for Eye {
    fn choose(&self, skull: &Skull) -> ConcreteAsset {
        let x = (skull.eyeball_left.0).0;
        let y = (skull.eyeball_left.0).1;
        let r = skull.eyeball_left.1;
        let mut svg = ellipse(x, y, r, r, "eye_color_1");
        svg.push_str(&ellipse(x, y, r*0.58, r*0.58, "eye_color_2"));
        svg.push_str(&ellipse(x, y, r*0.26, r*0.26, "eye_color_3"));

        let x = (skull.eyeball_right.0).0;
        let y = (skull.eyeball_right.0).1;
        let r = skull.eyeball_right.1;
        svg.push_str(&ellipse(x, y, r, r, "eye_color_1"));
        svg.push_str(&ellipse(x, y, r*0.58, r*0.58, "eye_color_2"));
        svg.push_str(&ellipse(x, y, r*0.26, r*0.26, "eye_color_3"));

        Box::new((
            SVGFragment {
                contents: svg,
                layer: self.front_layer,
            },
            None,
        ))
    }
}

fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, c: &str) -> String {
    format!(r#"<ellipse
style="fill:{};fill-opacity:1;"
cx="{}"
cy="{}"
rx="{}"
ry="{}" />
"#, c, cx, cy, rx, ry)
}

fn lids(cx: f64, cy: f64, rx: f64, ry: f64, c: &str) -> String {
    format!(r#"<svg:path
       style="fill:{};fill-opacity:1;"
       d="M {},{} a {},{} 0 1,0 {},-{} z"/>"#, c, cx-rx, cy-ry, rx, ry, rx, ry)
}
