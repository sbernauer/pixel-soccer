use lazy_static::lazy_static;
use regex::Regex;
use std::io::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
};

use crate::protocol::{PixelflutRequest, PixelflutResponse, Serialize};

lazy_static! {
    // Thanks to https://github.com/timvisee/pixelpwnr/blob/0d83b3e0b54448a59844e330a36f2e4b0e19e611/src/pix/client.rs#L19
    pub static ref SIZE_COMMAND_REGEX: Regex = Regex::new(r"^(?i)\s*SIZE\s+([[:digit:]]+)\s+([[:digit:]]+)\s*$").unwrap();
    pub static ref READ_PIXEL_COMMAND_REGEX: Regex = Regex::new(r"PX ([0-9]+) ([0-9]+) ([0-9a-fA-F]+)\s").unwrap();
}

pub struct Client {
    stream: BufStream<TcpStream>,
}

impl Client {
    pub async fn new(server_address: &str) -> Result<Self> {
        let stream = TcpStream::connect(server_address).await?;
        Ok(Client {
            stream: BufStream::new(stream),
        })
    }

    pub async fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Slow. For best performance use [write_bytes][Self::write_bytes]
    pub async fn write_commands(&mut self, commands: &Vec<PixelflutRequest>) -> Result<()> {
        let mut bytes = Vec::new();
        for command in commands {
            command.serialize(&mut bytes);
        }

        self.write_bytes(&bytes).await?;

        Ok(())
    }

    pub async fn read_commands(
        &mut self,
        number_of_commands: usize,
    ) -> Result<Vec<PixelflutResponse>> {
        let mut result = Vec::new();
        for _ in 0..number_of_commands {
            let mut buffer = String::new();
            self.stream.read_line(&mut buffer).await?;

            if let Some(matches) = READ_PIXEL_COMMAND_REGEX.captures(&buffer) {
                let x = matches[1]
                    .parse::<u16>()
                    .expect("Failed to parse x coordinate, received malformed data");
                let y = matches[2]
                    .parse::<u16>()
                    .expect("Failed to parse y coordinate, received malformed data");
                if let Ok(rgb) = u32::from_str_radix(matches.get(3).unwrap().as_str(), 16) {
                    result.push(PixelflutResponse::Pixel { x, y, rgb });
                }
            } else if let Some(matches) = SIZE_COMMAND_REGEX.captures(&buffer) {
                let width = matches[1]
                    .parse::<u16>()
                    .expect("Failed to parse screen width, received malformed data");
                let height = matches[2]
                    .parse::<u16>()
                    .expect("Failed to parse screen height, received malformed data");
                result.push(PixelflutResponse::Size { width, height });
            }
        }

        Ok(result)
    }

    pub async fn get_screen_size(&mut self) -> Result<(u16, u16)> {
        self.write_commands(&vec![PixelflutRequest::GetSize])
            .await?;
        let response = self.read_commands(1).await?;

        if let Some(PixelflutResponse::Size { width, height }) = response.get(0) {
            Ok((*width, *height))
        } else {
            panic!("Expected to get the size of the screen, but got {response:?}")
        }
    }
}
