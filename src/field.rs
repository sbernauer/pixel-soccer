use async_trait::async_trait;
use image::io::Reader as ImageReader;
use rand::{prelude::SliceRandom, thread_rng};

use crate::{
    client::{self, Client},
    draw::Draw,
    image_helpers,
};
use std::io::Result;

pub struct Field {
    draw_command_bytes: Vec<u8>,
}

impl Field {
    pub fn new() -> Self {
        let image = ImageReader::open("images/field_v3_1280.png")
            .unwrap()
            .decode()
            .unwrap();

        let mut draw_commands = image_helpers::draw_image(&image, 0, 0);

        // Shuffle commands to prevent drawing artefacts
        draw_commands.shuffle(&mut thread_rng());

        Self {
            draw_command_bytes: client::commands_to_bytes(&draw_commands),
        }
    }
}

#[async_trait]
impl Draw for Field {
    async fn draw(&self, client: &mut Client) -> Result<()> {
        client.write_bytes(&self.draw_command_bytes).await?;
        Ok(())
    }
}
