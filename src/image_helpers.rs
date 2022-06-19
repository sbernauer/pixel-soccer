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
