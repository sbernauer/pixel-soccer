use async_trait::async_trait;
use image::io::Reader as ImageReader;

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
        let image = ImageReader::open("images/field_v3.png")
            .unwrap()
            .decode()
            .unwrap();

        let draw_commands = image_helpers::draw_image(&image, 0, 0);

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
