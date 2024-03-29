use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use regex::Regex;

use resvg::svgdom::{
    AttributeId, AttributeValue, Color, Document, ElementId, FilterSvg, Node, NodeType,
    ParseOptions, PathSegment, Attribute, ViewBox,
};
use resvg::usvg;

use super::{GenerationContext, Guide, Palette};

pub struct Template {
    guides: Vec<(String, String, Guide, usize)>,
    optional_nodes: Vec<(String, usize)>,
    contents: Document,
    outer_guide: Option<Guide>,
}

impl Template {
    pub fn new(tree: &Node, outer_guide: Option<Guide>) -> Self {
        let mut doc = Document::new();
        doc.root().append(doc.copy_node_deep(tree.clone()));
        let mut guides = Vec::new();
        let mut optional_nodes = Vec::new();
        for (i, node) in tree.descendants().enumerate() {
            if node.has_id() {
                let id = node.id();
                if id.starts_with("guide_") {
                    let re = Regex::new(r"guide_(?P<name>[^-]+)").unwrap();
                    let caps = re.captures(&id).unwrap();
                    let feature_name = &caps["name"];
                    let vidx = feature_name.rfind(':').unwrap_or(feature_name.len());
                    let variant = feature_name[vidx..].to_owned();
                    let feature_name = feature_name[..vidx].to_owned();
                    let mut guide = Guide::new(&node);
                    if feature_name == "skull" {
                        fiddle_guide(&mut guide, (-0.1, 0.1), (-0.15, 0.08), (-0.15, 0.15), (0.0, 0.0));
                    } else if feature_name == "nose" {
                        fiddle_guide(&mut guide, (0.0, 0.0), (-0.05, 0.7), (0.0, 0.0), (-0.08, 0.25));
                    }
                    guides.push((feature_name, variant, guide, i));
                } else if id.starts_with("option_") {
                    let re = Regex::new(r"option_(?P<name>[^:-]+)").unwrap();
                    let caps = re.captures(&id).unwrap();
                    let feature_name = &caps["name"];
                    optional_nodes.push((feature_name.to_owned(), i));
                }
            }
        }
        Self {
            guides,
            contents: doc,
            optional_nodes,
            outer_guide,
        }
    }

    pub fn from_directory(path: &Path) -> HashMap<String, Self> {
        let mut results = HashMap::new();
        for entry in path.read_dir().unwrap() {
            if let Ok(entry) = entry {
                let name = entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_owned()
                    .into_string()
                    .unwrap();
                results.insert(name, Self::from_file(&entry.path()));
            }
        }
        results
    }

    pub fn from_file(path: &Path) -> Self {
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

        let mut classes = HashSet::new();
        for node in doc.root().descendants().filter(|node| node.has_attribute(AttributeId::Class)) {
            let attributes = node.attributes();
            let class_str = attributes.get_value(AttributeId::Class);
            if let Some(AttributeValue::String(class_str)) = class_str {
                for class in class_str.split(' ') {
                    classes.insert(class.to_string());
                }
            }
        }

        let mut css = doc.create_element("style");
        let rules:Vec<String> = classes.iter().map(|k| {
            let mut s = DefaultHasher::new();
            k.hash(&mut s);
            let h = s.finish() as u32;
            //TODO I have to be able to do this with just the formatting string
            let h = format!("#{:x}", h)[..7].to_string();
            if k.ends_with("_outline") {
                format!(".{} {{ stroke: {}; }}", k, h)
            } else {
                format!(".{} {{ fill: {}; }}", k, h)
            }
        }).collect();
        let text = rules.join("\n");
        let text = doc.create_node(
            NodeType::Text,
            text,
        );
        css.append(text);
        doc.svg_element().unwrap().prepend(css);

        let doc = resvg::usvg::Tree::from_str(
            &format!("{}", doc),
            &resvg::usvg::Options {
                keep_named_groups: true,
                ..resvg::usvg::Options::default()
            },
        )
        .unwrap()
        .to_svgdom();

        let mut guide = None;
        let mut template_node = None;
        for node in doc.root().descendants() {
            if node.has_id() {
                let id = node.id().clone();
                if id == "guide" {
                    guide = Some(Guide::new(&node.first_child().unwrap()));
                } else if id == "contents" {
                    template_node = Some(node.clone());
                }
            }
        }
        Self::new(&template_node.unwrap(), guide)
    }

    pub fn generate_from_context(&self, context: &GenerationContext, path: &str) -> Document {
        let mut doc = Document::new();
        let mut svg = doc.create_element(ElementId::Svg);
        let total_width = 210.0;
        let total_height = 210.0;
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
        let mut non_distort_nodes = Vec::new();
        let mut main_node = self.rec_generate_from_context(context, path, &mut non_distort_nodes, &mut doc);
        for (mut contents, sub_template, mut node) in non_distort_nodes {
            sub_template.align_contents(
                &mut contents,
                &Guide::new(&node),
                context.palette,
            );
            node.insert_after(contents);
            node.detach();
        }
        svg.append(main_node);
        doc.root().append(svg);
        doc
    }

    fn rec_generate_from_context<'a>(&'a self, context: &'a GenerationContext, path: &str, non_distort_nodes: &mut Vec<(Node, &'a Template, Node)>, doc: &mut Document) -> Node {
        let svg = doc.copy_node_deep(self.contents.root().first_child().unwrap());
        let mut nodes: Vec<Node> = svg.descendants().collect();

        for (name, node_idx) in &self.optional_nodes {
            if !context.use_optional(path, name) {
                nodes[*node_idx].detach();
            }
        }

        for (name, name_variant, guide, node_idx) in &self.guides {
            let sub_template = context.choose_template(path, name, name_variant);
            if let Some((sub_template, child_path)) = sub_template {
                let mut contents = sub_template.rec_generate_from_context(context, &child_path, non_distort_nodes, doc);
                if let Some(Guide::CircleGuide {..}) = sub_template.outer_guide {
                    non_distort_nodes.push((contents, &sub_template, nodes[*node_idx].clone()));
                } else {
                    sub_template.align_contents(
                        &mut contents,
                        guide,
                        context.palette,
                    );
                    nodes[*node_idx].insert_after(contents);
                    nodes[*node_idx].detach();
                }
            } else {
                nodes[*node_idx].detach();
            }
        }
        let mut g = doc.create_element(ElementId::G);
        for child in svg.children() {
            g.append(child);
        }
        g
    }

    pub fn align_contents(
        &self,
        node: &mut Node,
        target: &Guide,
        palette: &Palette,
    ) {
        apply_palette(node, palette);
        match self.outer_guide.unwrap() {
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
                            node,
                        );
                    }
                    _ => panic!(),
                }
            }
            Guide::CircleGuide { cx, cy, r } => {
                let (acx, acy, ar) = (cx, cy, r);
                if let Guide::CircleGuide { cx, cy, r } = target {
                    self.transformation_from_quad(
                        (
                            cx - r,
                            cy + r,

                            cx + r,
                            cy + r,

                            cx + r,
                            cy - r,

                            cx - r,
                            cy - r,
                        ),
                        (
                            acx - ar,
                            acy + ar,
                            acx + ar,
                            acy + ar,
                            acx + ar,
                            acy - ar,
                            acx - ar,
                            acy - ar,
                        ),
                        node,
                    );
                } else {
                    panic!();
                }
            }
        }
    }

    fn transformation_from_quad(
        &self,
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

fn apply_palette(root: &mut Node, palette: &Palette) {
    let palette_placeholders:HashMap<String, String> =
        HashMap::from_iter(palette.keys().map(|k| {
            let mut s = DefaultHasher::new();
            k.hash(&mut s);
            let h = s.finish() as u32;
            (format!("#{:x}", h)[..7].to_string(), k.to_string())
        }));

    for mut node in root.descendants() {
        let mut attrs = node.attributes_mut();
        for aid in &[AttributeId::Fill, AttributeId::Stroke] {
            if let Some(a) = attrs.get_mut(*aid) {
                let mut new_value = None;
                if let AttributeValue::Color(c) = a.value {
                    let as_text = c.to_string();
                    if palette_placeholders.contains_key(&as_text) {
                        new_value = Some(palette[&palette_placeholders[&as_text]].to_string());
                    }
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
                        }
                        | PathSegment::LineTo {
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


fn fiddle_guide(guide: &mut Guide, width_amount: (f64, f64), height_amount: (f64, f64), top_amount: (f64, f64), bottom_amount: (f64, f64)) {
   match guide {
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
           let iay = *ay;
           let iby = *by;
           let icy = *cy;
           let idy = *dy;
           let y_for_i = |i| match i {
                   0 => iay,
                   1 => iby,
                   2 => icy,
                   _ => idy,
           };
           let x_for_i = |i| match i {
                   0 => *ax,
                   1 => *bx,
                   2 => *cx,
                   _ => *dx,
           };

           let mut by_y = [0, 1, 2, 3];
           by_y.sort_by_key(|i| (y_for_i(*i) * 10000.0) as i32);
           let mut by_x = [0, 1, 2, 3];
           by_x.sort_by_key(|i| (x_for_i(*i) * 10000.0) as i32);

           if width_amount.0 != 0.0 || width_amount.1 != 0.0 {
               let width = x_for_i(by_x[3]) - x_for_i(by_x[0]);
               let amount = (width * thread_rng().gen_range(width_amount.0, width_amount.1)) / 2.0;
               for (i, a) in &[(0, -amount), (1, -amount), (2, amount), (3, amount)] {
                   match by_x[*i] {
                       0 => *ax += a,
                       1 => *bx += a,
                       2 => *cx += a,
                       _ => *dx += a,
                   };
               }
           }

           if height_amount.0 != 0.0 || height_amount.1 != 0.0 {
               let height = y_for_i(by_y[3]) - y_for_i(by_y[0]);
               let amount = (height * thread_rng().gen_range(height_amount.0, height_amount.1)) / 2.0;
               for (i, a) in &[(0, amount), (1, amount)] {
                   match by_y[*i] {
                       0 => *ay += a,
                       1 => *by += a,
                       2 => *cy += a,
                       _ => *dy += a,
                   };
               }
           }

           if top_amount.0 != 0.0 || top_amount.1 != 0.0 {
               let height = y_for_i(by_y[3]) - y_for_i(by_y[0]);
               let amount = (height * thread_rng().gen_range(top_amount.0, top_amount.1)) / 2.0;
               for (i, a) in &[(0, amount), (1, -amount)] {
                   match by_y[*i] {
                       0 => *ax += a,
                       1 => *bx += a,
                       2 => *cx += a,
                       _ => *dx += a,
                   };
               }
           }

           if bottom_amount.0 != 0.0 || bottom_amount.1 != 0.0 {
               let height = y_for_i(by_y[3]) - y_for_i(by_y[0]);
               let amount = (height * thread_rng().gen_range(bottom_amount.0, bottom_amount.1)) / 2.0;
               for (i, a) in &[(2, amount), (3, -amount)] {
                   match by_y[*i] {
                       0 => *ax += a,
                       1 => *bx += a,
                       2 => *cx += a,
                       _ => *dx += a,
                   };
               }
           }


       },
       _ => (),
   }
}
