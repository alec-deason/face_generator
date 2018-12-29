use rand::Rng;
use std::collections::HashMap;

use super::Pallete;

pub fn generate_pallete() -> Pallete {
    let mut rng = rand::thread_rng();

    let mut pallete = HashMap::new();

    let base_skin_color = (rng.gen_range(21.0, 35.0), 163.0, rng.gen_range(50.0, 156.0));
    let is_pale_complexion = base_skin_color.2 > 120.0;

    let base_skin_color = (
        base_skin_color.0 / 256.0,
        base_skin_color.1 / 256.0,
        base_skin_color.2 / 256.0,
    );

    let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2);
    pallete.insert("skin_color".to_string(), format!("#{:01$x}", rgb, 6));

    let rgb = hslToRgb(base_skin_color.0, base_skin_color.1, base_skin_color.2*0.6);
    pallete.insert("skin_color_dark".to_string(), format!("#{:01$x}", rgb, 6));

    // TODO: Martin scale!
    let base_eye_color;
    match rng.gen_range(0, if is_pale_complexion { 4 } else { 2 }) {
        0 => {
            // Dark brown
            base_eye_color = (rng.gen_range(18.0, 27.0), 222.0, rng.gen_range(50.0, 82.0));
        }
        2 => {
            // Hazel
            base_eye_color = (rng.gen_range(18.0, 27.0), 222.0, rng.gen_range(60.0, 92.0));
        }
        3 => {
            // Blue
            base_eye_color = (
                rng.gen_range(150.0, 160.0),
                161.0,
                rng.gen_range(104.0, 160.0),
            );
        }
        _ => {
            // Green
            base_eye_color = (
                rng.gen_range(70.0, 90.0),
                161.0,
                rng.gen_range(104.0, 160.0),
            );
        }
    }

    let base_eye_color = (
        base_eye_color.0 / 256.0,
        base_eye_color.1 / 256.0,
        base_eye_color.2 / 256.0,
    );
    let rgb = hslToRgb(base_eye_color.0, base_eye_color.1, base_eye_color.2);
    pallete.insert("eye_color".to_string(), format!("#{:01$x}", rgb, 6));

    let base_hair_color;
    match rng.gen_range(0, if is_pale_complexion { 4 } else { 1 }) {
        0 => {
            // Black Hair
            base_hair_color = (rng.gen_range(18.0, 27.0), 222.0, rng.gen_range(4.0, 20.0));
        }
        1 => {
            // Brown Hair
            base_hair_color = (rng.gen_range(18.0, 27.0), 222.0, rng.gen_range(50.0, 82.0));
        }
        2 => {
            // Red hair
            base_hair_color = (rng.gen_range(6.0, 15.0), 222.0, rng.gen_range(100.0, 140.0));
        }
        _ => {
            // Blond hair
            base_hair_color = (
                rng.gen_range(29.0, 40.0),
                222.0,
                rng.gen_range(100.0, 150.0),
            );
        }
    }

    let base_hair_color = (
        base_hair_color.0 / 256.0,
        base_hair_color.1 / 256.0,
        base_hair_color.2 / 256.0,
    );
    let rgb = hslToRgb(base_hair_color.0, base_hair_color.1, base_hair_color.2);
    pallete.insert("hair_color".to_string(), format!("#{:01$x}", rgb, 6));

    pallete
}

fn hslToRgb(h: f64, s: f64, l: f64) -> u32 {
    let r;
    let g;
    let b;

    if s == 0.0 {
        // achromatic
        r = l;
        g = l;
        b = l;
    } else {
        let hue2rgb = |p, q, mut t| {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.9;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0 / 2.0 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        r = hue2rgb(p, q, h + 1.0 / 3.0);
        g = hue2rgb(p, q, h);
        b = hue2rgb(p, q, h - 1.0 / 3.0);
    }

    let mut rgb = (r * 255.0) as u32;
    rgb = (rgb << 8) + (g * 255.0) as u32;
    rgb = (rgb << 8) + (b * 255.0) as u32;
    rgb
}
