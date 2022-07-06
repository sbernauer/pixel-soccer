use crate::protocol::PixelflutRequest;

use image::{DynamicImage, GenericImageView};

#[allow(dead_code)]
pub fn draw_rect(
    x_offset: u16,
    y_offset: u16,
    width: u16,
    height: u16,
    rgb: u32,
) -> Vec<PixelflutRequest> {
    let mut result = Vec::with_capacity(width as usize * height as usize);

    for x in x_offset..x_offset + width {
        for y in y_offset..y_offset + height {
            result.push(PixelflutRequest::SetPixel { x, y, rgb });
        }
    }

    result
}

pub fn draw_image(image: &DynamicImage, x_offset: u16, y_offset: u16) -> Vec<PixelflutRequest> {
    let mut result = Vec::with_capacity(image.width() as usize * image.height() as usize);

    for x in 0..image.width() as u16 {
        for y in 0..image.height() as u16 {
            match image.get_pixel(x as u32, y as u32).0 {
                [_, _, _, 0] => (), // Don't draw transparent pixels
                [r, g, b, _] => result.push(PixelflutRequest::SetPixel {
                    x: x_offset + x,
                    y: y_offset + y,
                    rgb: (r as u32) << 16 | (g as u32) << 8 | b as u32,
                }),
            }
        }
    }

    result
}

/// `x_center` and `y_center` are allowed to be negative or too high, so that the screen bounds are exceeded.
/// This function will handle that cases and not include invalid coordinates.
pub fn get_donut_coordinates(
    x_center: i16,
    y_center: i16,
    inner_circle_radius: f32,
    outer_circle_radius: f32,
    screen_width: u16,
    screen_height: u16,
) -> Vec<(u16, u16)> {
    let mut donut_coordinates = Vec::new();
    for x in x_center - outer_circle_radius as i16..x_center + outer_circle_radius as i16 {
        for y in y_center - outer_circle_radius as i16..y_center + outer_circle_radius as i16 {
            if x >= 0 && x < screen_width as i16 && y >= 0 && y < screen_height as i16 {
                let x_rel = (x - x_center) as f32;
                let y_rel = (y - y_center) as f32;
                let distance = f32::sqrt(f32::powi(x_rel, 2) + f32::powi(y_rel, 2));

                if distance >= inner_circle_radius && distance <= outer_circle_radius {
                    donut_coordinates.push((x as u16, y as u16));
                }
            }
        }
    }

    donut_coordinates
}
