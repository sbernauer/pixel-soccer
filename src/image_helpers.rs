use crate::protocol::PixelflutRequest;

use image::{DynamicImage, GenericImageView};
use rusttype::{point, Font, Scale};

pub const WHITE: u32 = 0x00ff_ffff;
pub const BLACK: u32 = 0x0000_0000;
pub const RED: u32 = 0x00ff_0000;

#[allow(dead_code)]
pub fn draw_rect(
    x_offset: u16,
    y_offset: u16,
    width: u16,
    height: u16,
    rgb: u32,
) -> Vec<PixelflutRequest> {
    let mut draw_commands = Vec::with_capacity(width as usize * height as usize);

    for x in x_offset..x_offset + width {
        for y in y_offset..y_offset + height {
            draw_commands.push(PixelflutRequest::SetPixel { x, y, rgb });
        }
    }

    draw_commands
}

pub fn draw_image(image: &DynamicImage, x_offset: u16, y_offset: u16) -> Vec<PixelflutRequest> {
    let mut draw_commands = Vec::with_capacity(image.width() as usize * image.height() as usize);

    for x in 0..image.width() as u16 {
        for y in 0..image.height() as u16 {
            match image.get_pixel(x as u32, y as u32).0 {
                [_, _, _, 0] => (), // Don't draw transparent pixels
                [r, g, b, _] => draw_commands.push(PixelflutRequest::SetPixel {
                    x: x_offset + x,
                    y: y_offset + y,
                    rgb: (r as u32) << 16 | (g as u32) << 8 | b as u32,
                }),
            }
        }
    }

    draw_commands
}

pub fn draw_text(
    x: u16,
    y: u16,
    scale: f32,
    color: u32,
    text: &str,
    font: &Font,
) -> Vec<PixelflutRequest> {
    let mut draw_commands = Vec::new();

    let scale = Scale::uniform(scale);
    let v_metrics = font.v_metrics(scale);

    let glyphs = font.layout(text, scale, point(x as f32, y as f32 + v_metrics.ascent));

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                if v > 0.5 {
                    draw_commands.push(PixelflutRequest::SetPixel {
                        x: x as u16 + bounding_box.min.x as u16,
                        y: y as u16 + bounding_box.min.y as u16,
                        rgb: color,
                    })
                }
            });
        }
    }

    draw_commands
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_with_background(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    scale: f32,
    text_color: u32,
    background_color: u32,
    text: &str,
    font: &Font,
) -> Vec<PixelflutRequest> {
    println!("Drawing text {text}");
    let mut draw_commands = draw_text(x, y, scale, text_color, text, font);

    let mut pixels = vec![vec![background_color; height as usize]; width as usize];
    for command in &draw_commands {
        if let PixelflutRequest::SetPixel {
            x: x_abs,
            y: y_abs,
            rgb,
        } = command
        {
            let x_rel = x_abs - x;
            let y_rel = y_abs - y;
            if x_rel < width && y_rel < height {
                pixels[x_rel as usize][y_rel as usize] = *rgb;
            }
        }
    }

    for x_abs in x..x + width {
        for y_abs in y..y + height {
            if pixels[(x_abs - x) as usize][(y_abs - y) as usize] == background_color {
                draw_commands.push(PixelflutRequest::SetPixel {
                    x: x_abs,
                    y: y_abs,
                    rgb: background_color,
                })
            }
        }
    }
    draw_commands
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
