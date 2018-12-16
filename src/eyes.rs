use super::{AbstractAssetTrait, ConcreteAssetTrait, ConcreteAsset, SVGFragment, Skull};


pub struct Eye {
    pub front_layer: u32,
    pub eyelids: Box<AbstractAssetTrait>,
}

impl AbstractAssetTrait for Eye {
    fn choose(&self, skull: &Skull) -> ConcreteAsset {
        let x = (skull.eyeball_left.0).0;
        let y = (skull.eyeball_left.0).1;
        let r = skull.eyeball_left.1;
        let mut svg = ellipse(x, y, r, r, "eye_white_color");
        svg.push_str(&ellipse(x, y, r*0.58, r*0.58, "eye_color"));
        svg.push_str(&ellipse(x, y, r*0.26, r*0.26, "eye_pupil_color"));

        let x = (skull.eyeball_right.0).0;
        let y = (skull.eyeball_right.0).1;
        let r = skull.eyeball_right.1;
        svg.push_str(&ellipse(x, y, r, r, "eye_white_color"));
        svg.push_str(&ellipse(x, y, r*0.58, r*0.58, "eye_color"));
        svg.push_str(&ellipse(x, y, r*0.26, r*0.26, "eye_pupil_color"));
        svg.push_str(&self.eyelids.choose(skull).to_svg_fragments()[0].contents);

        Box::new((
            SVGFragment {
                contents: svg,
                layer: self.front_layer,
            },
            None,
        ))
    }
}

fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, css_class: &str) -> String {
    format!(r#"<ellipse
style="fill-opacity:1;"
class="{}"
cx="{}"
cy="{}"
rx="{}"
ry="{}" />
"#, css_class, cx, cy, rx, ry)
}

fn lids(cx: f64, cy: f64, rx: f64, ry: f64, c: &str) -> String {
    format!(r#"<svg:path
       style="fill:{};fill-opacity:1;"
       d="M {},{} a {},{} 0 1,0 {},-{} z"/>"#, c, cx-rx, cy-ry, rx, ry, rx, ry)
}
