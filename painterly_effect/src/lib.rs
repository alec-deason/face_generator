extern crate image;
extern crate rand;
extern crate rstar;

use rstar::{RTree, RTreeObject, AABB, PointDistance};

use std::fs::File;

use std::collections::HashMap;
use std::f32;
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
pub fn apply_crystalization_effect(img: &im::ImageBuffer<im::Rgba<u8>, Vec<u8>>) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let mut rng = thread_rng();

    let width = img.width();
    let height = img.height();
    let far_density = 0.005;
    let near_density = 1.0;

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
                (x, y, p, (total_diff / (256.0*3.0*9.0)).powf(0.25))
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
            //p.blend(&im::Rgba([0, 0, 0, a]));
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

struct Brush {
    x: f32,
    y: f32,
    a: f32,
    goop: f32,
    color: im::Rgba<u8>,
}

pub fn apply_brush_agents_effect(img: &im::ImageBuffer<im::Rgba<u8>, Vec<u8>>) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let mut rng = thread_rng();

    let width = img.width();
    let height = img.height();

    let mut by_detail:Vec<(u32, u32)> = (0..width).map(|x| (0..height).map(move |y| (x, y))).flatten().collect();
    by_detail.shuffle(&mut rng);
    by_detail.sort_unstable_by_key(|(x, y)| {
        let mut total_diff = 0.0;
        let pc = img.get_pixel(*x, *y).channels();
        for xx in (*x as i32 - 1).max(0)..(*x as i32+2).min(width as i32) {
            for yy in (*y as i32 - 1).max(0)..(*y as i32+2).min(height as i32) {
                let oc = img.get_pixel(xx as u32, yy as u32).channels();
                total_diff += (oc[0] as f32- pc[0] as f32).abs();
                total_diff += (oc[1] as f32- pc[1] as f32).abs();
                total_diff += (oc[2] as f32- pc[2] as f32).abs();
            }
        }
        total_diff = 1.0 - total_diff / (256.0*3.0*9.0);
        (total_diff * 1000000.0) as i32
    });


    let mut new_img:im::ImageBuffer<im::Rgba<u8>, Vec<u8>> = im::ImageBuffer::new(width, height);
    let brush_scales = [
        (10i32, 0.004, 100, 0.0, 1.0),
        (6, 0.002, 30, 0.0, 0.5),
        (3, 0.002, 5, 0.0, 0.1),
        (2, 0.01, 5, 0.0, 0.1),
        (1, 0.01, 2, 0.0, 0.1)
    ];
    for (i, (brush_radius, density, steps, start, stop)) in brush_scales.iter().enumerate() {
        let mut brush_pattern = Vec::with_capacity((brush_radius*2  ).pow(2) as usize);
        for x in -brush_radius..(brush_radius+1) {
            for y in -brush_radius..(brush_radius+1) {
                let d = ((x as f32).powf(2.0) + (y as f32).powf(2.0)).sqrt();
                if d <= *brush_radius as f32 {
                    brush_pattern.push((x, y, rng.gen_range(0.5, 1.0)));
                }
            }
        }
        let agent_count = (width as f32 * height as f32 * density) as usize;
        let mut agents = Vec::with_capacity(agent_count);

        let start = by_detail.len() as f32 * start;
        let end = by_detail.len() as f32 * stop;

        for (x, y) in by_detail[start as usize..end as usize].choose_multiple(&mut rng, agent_count) {
            let mut color = *img.get_pixel(*x as u32, *y as u32);
            let mut channels = color.channels_mut();
            channels[0] = (channels[0] as f32 * rng.gen_range(0.6, 1.5)).min(255.0) as u8;
            channels[1] = (channels[1] as f32 * rng.gen_range(0.6, 1.5)).min(255.0) as u8;
            channels[2] = (channels[2] as f32 * rng.gen_range(0.6, 1.5)).min(255.0) as u8;
            channels[3] = 50;
            let a = rng.gen_range(0.0, f32::consts::PI * 2.0);
            agents.push(Brush { x: *x as f32, y: *y as f32, a, goop: 1.0, color });
        }
        for _ in 0..*steps {
            for brush in &mut agents {
                brush.x = (brush.x + brush.a.cos()).max(0.0).min(width as f32 - 1.0);
                brush.y = (brush.y + brush.a.sin()).max(0.0).min(height as f32 - 1.0);
                brush.a += rng.gen_range(-0.5, 0.5);
                let mut color = *img.get_pixel(brush.x as u32, brush.y as u32);
                let mut channels = color.channels_mut();
                channels[0] = (channels[0] as f32 * rng.gen_range(0.9, 1.1)).min(255.0) as u8;
                channels[1] = (channels[1] as f32 * rng.gen_range(0.9, 1.1)).min(255.0) as u8;
                channels[2] = (channels[2] as f32 * rng.gen_range(0.9, 1.1)).min(255.0) as u8;
                channels[3] = 50;
                brush.color.blend(&color);

                for (x, y, strength) in &brush_pattern {
                    let x = (*x as f32 + brush.x);
                    let y = (*y as f32 + brush.y);
                    if x >= 0.0 && y >= 0.0 && x < width as f32 && y < height as f32 {
                        let mut color = brush.color.clone();
                        let mut channels = color.channels_mut();
                        channels[3] = (channels[3] as f32 * strength * brush.goop.powf(0.5)) as u8;
                        new_img.get_pixel_mut(x as u32, y as u32).blend(&color);
                    }
                }
                brush.goop -= 1.0/ *steps as f32;
            }
        }
    }

    new_img
}
