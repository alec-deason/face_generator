extern crate image;
extern crate rand;
extern crate rstar;

use rstar::{RTree, RTreeObject, AABB, PointDistance};

use rand::prelude::*;
use image as im;
use image::Pixel;

#[derive(Clone)]
struct GridCell {
    x: f32,
    y: f32,
    value: im::Rgba<u8>,
}
impl RTreeObject for GridCell {
    type Envelope = AABB<[f32; 2]>;
    fn envelope(&self) -> Self::Envelope
    {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for GridCell {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        let d_x = self.x - point[0];
        let d_y = self.y - point[1];
        let distance = (d_x * d_x + d_y * d_y).sqrt();
        distance * distance
    }
}

//TODO blaa blaa generic
pub fn apply_effect(img: &im::ImageBuffer<im::Rgba<u8>, Vec<u8>>) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let mut rng = thread_rng();

    let width = img.width();
    let height = img.height();
    let far_density = 0.005;
    let near_density = 5.0;

    let mut scribblies = Vec::new();
    for _ in 0..6 {
        scribblies.push(RTree::bulk_load(
            img.enumerate_pixels().map(|(x, y, p)| {
                let mut total_diff = 0.0;
                for xx in (x as i32 - 1).max(0)..(x as i32+2).min(width as i32) {
                    for yy in (y as i32 - 1).max(0)..(y as i32+2).min(height as i32) {
                        let oc = img.get_pixel(xx as u32, yy as u32).channels();
                        let pc = p.channels();
                        total_diff += (oc[0] as f32- pc[0] as f32).abs();
                        total_diff += (oc[1] as f32- pc[1] as f32).abs();
                        total_diff += (oc[2] as f32- pc[2] as f32).abs();
                    }
                }
                (x, y, p, total_diff / (256.0*3.0*9.0))
            }).filter(|(_, _, _, d)| {
                let density = (near_density - far_density) * d + far_density;
                thread_rng().gen::<f32>() < density
            }).map(|(x, y, p, d)| {
                    let mut p = *p;
                    let a = rng.gen_range(0, ((1.0-d)*50.0) as u8);
                    p.blend(&im::Rgba([0, 0, 0, a]));
                    GridCell {
                        x: x as f32,
                        y: y as f32,
                        value: p
                    }
            }).collect()));
    }

    let mut new_img = im::ImageBuffer::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let mut p = scribblies[0].nearest_neighbor(&[x as f32, y as f32]).unwrap().value;
            for scribbly in &scribblies[1..] {
                let mut pp = scribbly.nearest_neighbor(&[x as f32, y as f32]).unwrap().value;
                pp.channels_mut()[3] = (256.0/scribblies.len() as f32) as u8;
                p.blend(&pp);
            }
            let a = rng.gen_range(0, 20);
            p.blend(&im::Rgba([0, 0, 0, a]));
            /*
            let mut op = *img.get_pixel(x, y);
            op.channels_mut()[3] = 128;
            p.blend(&op);
            */
            
            new_img.put_pixel(x, y, p);
        }
    }
    new_img
}
