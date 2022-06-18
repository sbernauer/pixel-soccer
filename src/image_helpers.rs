use crate::protocol::PixelflutRequest;

use image::{DynamicImage, GenericImageView};

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

pub fn draw_coordinates(coordinates: Vec<(u16, u16)>, rgb: u32) -> Vec<PixelflutRequest> {
    coordinates
        .into_iter()
        .map(|(x, y)| PixelflutRequest::SetPixel { x, y, rgb })
        .collect()
}

pub fn coordinates_on_line(start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Vec<(u16, u16)> {
    let mut coordinates = vec![];
    let length =
        ((end_x - start_x) * (end_x - start_x) + (end_y - start_y) * (end_y - start_y)).sqrt();
    let x_step = (end_x - start_x) / length;
    let y_step = (end_y - start_y) / length;

    for step in 0..length as u16 {
        coordinates.push((
            (start_x + step as f32 * x_step) as u16,
            (start_y + step as f32 * y_step) as u16,
        ));
    }

    coordinates
}

pub fn coordinates_on_line_with_dir_and_skip_offset(
    x: f32,
    y: f32,
    dir: f32,
    length: f32,
    skip_offset: f32,
) -> Vec<(u16, u16)> {
    let start_x = x + skip_offset * dir.cos();
    let start_y = y + skip_offset * dir.sin();
    let end_x = x + length * dir.cos();
    let end_y = y + length * dir.sin();

    coordinates_on_line(start_x, start_y, end_x, end_y)
}
