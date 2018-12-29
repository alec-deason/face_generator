use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;

use std::str::FromStr;
use svgdom::{
    AttributeId, AttributeValue, Color, Document, ElementId, FilterSvg, Node, NodeType,
    ParseOptions, PathSegment, WriteBuffer,
};
use usvg;

use super::{Guide, Pallete};

pub struct Feature {
    guide: Guide,
    doc_ref: Rc<RefCell<Document>>,
    contents: Node,
}

impl Feature {
    pub fn all_from_file(path: &Path) -> Vec<Feature> {
        let mut features_svg = Vec::new();
        let mut guide = None;

        let mut file = File::open(path).unwrap();
        let length = file.metadata().unwrap().len() as usize;

        let mut input_data = String::with_capacity(length + 1);
        file.read_to_string(&mut input_data).unwrap();
        let mut doc = Document::from_str_with_opt(
            &input_data,
            &ParseOptions {
                skip_unresolved_classes: false,
                skip_invalid_css: true,
                ..ParseOptions::default()
            },
        )
        .unwrap();

        let mut css = doc.create_element("style");
        let text = doc.create_node(
            NodeType::Text,
            r#"
.skin_color { fill: #b1b2b3; }
.skin_color_outline { stroke: #b2b2b3; }
.eye_color { fill: #b2b3b4; }
.hair_color { fill: #b3b4b5; }
.hair_color_outline { stroke: #b4b4b5; }
        "#,
        );
        css.append(text);
        doc.svg_element().unwrap().prepend(css);

        let mut output_data = Vec::new();
        doc.write_buf(&mut output_data);

        let mut f = File::create("/tmp/feature_pre.svg").unwrap();
        f.write_all(&output_data).unwrap();

        let doc = usvg::Tree::from_str(
            &format!("{}", doc),
            &usvg::Options {
                keep_named_groups: true,
                ..usvg::Options::default()
            },
        )
        .unwrap();
        let mut output_data = Vec::new();
        doc.to_svgdom().write_buf(&mut output_data);

        let mut f = File::create("/tmp/feature_post.svg").unwrap();
        f.write_all(&output_data).unwrap();

        let doc = Rc::new(RefCell::new(doc.to_svgdom()));
        let mut doc_ref = doc.borrow_mut();
        for (_, node) in doc_ref.root().descendants().svg() {
            if node.has_id() {
                let id = node.id().clone();
                if id == "guide" {
                    guide = Some(Guide::new(&node.first_child().unwrap()));
                } else if id.starts_with("feature_") {
                    let contents = doc_ref.copy_node_deep(node);
                    features_svg.push(contents);
                }
            }
        }
        let guide = guide.unwrap();
        let result = features_svg
            .drain(..)
            .map(|f| Feature {
                guide,
                doc_ref: doc.clone(),
                contents: f,
            })
            .collect();
        result
    }

    pub fn aligned_contents(&mut self, target: &Guide, pallete: &Pallete) -> Node {
        let mut node = self
            .doc_ref
            .borrow_mut()
            .copy_node_deep(self.contents.clone());
        apply_pallete(&mut node, pallete);
        match self.guide {
            Guide::QuadGuide {
                ax,
                ay,
                bx,
                by,
                cx,
                cy,
                dx,
                dy,
            } => {
                let (axs, ays, bxs, bys, cxs, cys, dxs, dys) = (ax, ay, bx, by, cx, cy, dx, dy);
                match target {
                    Guide::QuadGuide {
                        ax,
                        ay,
                        bx,
                        by,
                        cx,
                        cy,
                        dx,
                        dy,
                    } => {
                        self.transformation_from_quad(
                            (*ax, *ay, *bx, *by, *cx, *cy, *dx, *dy),
                            (axs, ays, bxs, bys, cxs, cys, dxs, dys),
                            &mut node,
                        );
                    }
                    _ => panic!(),
                }
            }
            _ => panic!("Unimplemented"),
        }
        node
    }

    fn transformation_from_quad(
        &mut self,
        feat: (f64, f64, f64, f64, f64, f64, f64, f64),
        guide: (f64, f64, f64, f64, f64, f64, f64, f64),
        node: &mut Node,
    ) {
        let m = transform2d(
            guide.0, guide.1, guide.2, guide.3, guide.4, guide.5, guide.6, guide.7, feat.0, feat.1,
            feat.2, feat.3, feat.4, feat.5, feat.6, feat.7,
        );

        apply_matrix(node, &m);
    }
}

fn apply_pallete(root: &mut Node, pallete: &Pallete) {
    let skin_color: Color = Color::from_str("#b1b2b3").unwrap();
    let skin_color_dark: Color = Color::from_str("#b2b2b3").unwrap();
    let eye_color: Color = Color::from_str("#b2b3b4").unwrap();
    let hair_color: Color = Color::from_str("#b3b4b5").unwrap();
    let hair_color_outline: Color = Color::from_str("#b4b4b5").unwrap();

    for mut node in root.descendants() {
        let mut attrs = node.attributes_mut();
        for aid in &[AttributeId::Fill, AttributeId::Stroke] {
            if let Some(a) = attrs.get_mut(*aid) {
                let mut new_value = None;
                if let AttributeValue::Color(c) = a.value {
                    new_value = if c == skin_color {
                        let new_c = pallete["skin_color"].clone();
                        Some(new_c)
                    } else if c == skin_color_dark {
                        let new_c = pallete["skin_color_outline"].clone();
                        Some(new_c)
                    } else if c == eye_color {
                        let new_c = pallete["eye_color"].clone();
                        Some(new_c)
                    } else if c == hair_color {
                        let new_c = pallete["hair_color"].clone();
                        Some(new_c)
                    } else if c == hair_color_outline {
                        let new_c = pallete["hair_color_outline"].clone();
                        Some(new_c)
                    } else {
                        None
                    };
                }
                if new_value.is_some() {
                    a.value = AttributeValue::Color(Color::from_str(&new_value.unwrap()).unwrap());
                }
            }
        }
    }
}

fn mpoint(m: &[f64; 16], x: f64, y: f64) -> (f64, f64) {
    let a = x * m[0] + y * m[4] + m[12];
    let b = x * m[1] + y * m[5] + m[13];
    let w = x * m[3] + y * m[7] + m[15];

    let x = a / w;
    let y = b / w;
    (x, y)
}

fn apply_matrix(node: &mut Node, matrix: &[f64; 16]) {
    for (id, mut node) in node.descendants().svg() {
        if id == ElementId::Path {
            let mut attrs = node.attributes_mut();
            if let Some(&mut AttributeValue::Path(ref mut path)) =
                attrs.get_value_mut(AttributeId::D)
            {
                for seg in path.iter_mut() {
                    match *seg {
                        PathSegment::MoveTo {
                            ref mut x,
                            ref mut y,
                            ..
                        } => {
                            let (xx, yy) = mpoint(matrix, *x, *y);
                            *x = xx;
                            *y = yy;
                        }
                        PathSegment::LineTo {
                            ref mut x,
                            ref mut y,
                            ..
                        } => {
                            let (xx, yy) = mpoint(matrix, *x, *y);
                            *x = xx;
                            *y = yy;
                        }
                        PathSegment::CurveTo {
                            ref mut x1,
                            ref mut y1,
                            ref mut x2,
                            ref mut y2,
                            ref mut x,
                            ref mut y,
                            ..
                        } => {
                            let (xx, yy) = mpoint(matrix, *x, *y);
                            *x = xx;
                            *y = yy;
                            let (xx, yy) = mpoint(matrix, *x1, *y1);
                            *x1 = xx;
                            *y1 = yy;
                            let (xx, yy) = mpoint(matrix, *x2, *y2);
                            *x2 = xx;
                            *y2 = yy;
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn adj(m: [f64; 9]) -> [f64; 9] {
    // Compute the adjugate of m
    [
        m[4] * m[8] - m[5] * m[7],
        m[2] * m[7] - m[1] * m[8],
        m[1] * m[5] - m[2] * m[4],
        m[5] * m[6] - m[3] * m[8],
        m[0] * m[8] - m[2] * m[6],
        m[2] * m[3] - m[0] * m[5],
        m[3] * m[7] - m[4] * m[6],
        m[1] * m[6] - m[0] * m[7],
        m[0] * m[4] - m[1] * m[3],
    ]
}

fn multmm(a: [f64; 9], b: [f64; 9]) -> [f64; 9] {
    // multiply two matrices
    let mut c = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for i in 0..3 {
        for j in 0..3 {
            let mut cij = 0.0;
            for k in 0..3 {
                cij += a[3 * i + k] * b[3 * k + j];
            }
            c[3 * i + j] = cij;
        }
    }
    c
}

fn multmv(m: [f64; 9], v: [f64; 3]) -> [f64; 3] {
    // multiply matrix and vector
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

fn basis_to_points(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
) -> [f64; 9] {
    let m = [x1, x2, x3, y1, y2, y3, 1.0, 1.0, 1.0];
    let v = multmv(adj(m), [x4, y4, 1.0]);
    multmm(m, [v[0], 0.0, 0.0, 0.0, v[1], 0.0, 0.0, 0.0, v[2]])
}

fn general_2d_projection(
    x1s: f64,
    y1s: f64,
    x1d: f64,
    y1d: f64,
    x2s: f64,
    y2s: f64,
    x2d: f64,
    y2d: f64,
    x3s: f64,
    y3s: f64,
    x3d: f64,
    y3d: f64,
    x4s: f64,
    y4s: f64,
    x4d: f64,
    y4d: f64,
) -> [f64; 9] {
    let s = basis_to_points(x1s, y1s, x2s, y2s, x3s, y3s, x4s, y4s);
    let d = basis_to_points(x1d, y1d, x2d, y2d, x3d, y3d, x4d, y4d);
    multmm(d, adj(s))
}

fn transform2d(
    x1s: f64,
    y1s: f64,
    x2s: f64,
    y2s: f64,
    x3s: f64,
    y3s: f64,
    x4s: f64,
    y4s: f64,
    x1d: f64,
    y1d: f64,
    x2d: f64,
    y2d: f64,
    x3d: f64,
    y3d: f64,
    x4d: f64,
    y4d: f64,
) -> [f64; 16] {
    let mut t = general_2d_projection(
        x1s, y1s, x1d, y1d, x2s, y2s, x2d, y2d, x3s, y3s, x3d, y3d, x4s, y4s, x4d, y4d,
    );
    for i in 0..9 {
        t[i] /= t[8];
    }
    [
        t[0], t[3], 0.0, t[6], t[1], t[4], 0.0, t[7], 0.0, 0.0, 1.0, 0.0, t[2], t[5], 0.0, t[8],
    ]
}
