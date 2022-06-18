use core::panic;
use lazy_static::lazy_static;
use regex::Regex;
use std::{io::Result, vec};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
};

use crate::protocol::{PixelflutRequest, PixelflutResponse, Serialize};

pub const AVG_BYES_PER_PIXEL_SET_COMMAND: usize = "PX 123 123 ffffff\n".len();

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

            let mut parts = buffer.split(' ');
            match parts.next() {
                Some("PX") => {
                    let x = parts
                        .next()
                        .expect("invalid PX response - missing x coordinate")
                        .parse::<u16>()
                        .unwrap();
                    let y = parts
                        .next()
                        .expect("invalid PX response - missing y coordinate")
                        .parse::<u16>()
                        .unwrap();
                    let rgb = u32::from_str_radix(
                        parts
                            .next()
                            .expect("invalid PX response - missing rgb color")
                            .trim_end_matches('\n'),
                        16,
                    )
                    .unwrap();
                    result.push(PixelflutResponse::Pixel { x, y, rgb });
                }
                Some("SIZE") => {
                    let width = parts
                        .next()
                        .expect("invalid SIZE response - missing width")
                        .parse::<u16>()
                        .unwrap();
                    let height = parts
                        .next()
                        .expect("invalid SIZE response - missing height")
                        .trim_end_matches('\n')
                        .parse::<u16>()
                        .unwrap();
                    result.push(PixelflutResponse::Size { width, height });
                }
                None | Some(_) => panic!("Could not read response {buffer:?}"),
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

    /// `x_offset` and `y_offset` are allowed to be negative or too high, so that the screen bounds are exceeded.
    /// This function will handle that cases and fill the returned rectangle with 0s if they are out of bounds
    pub async fn get_screen_rect(
        &mut self,
        x_offset: i16,
        y_offset: i16,
        width: u16,
        height: u16,
        screen_width: u16,
        screen_height: u16,
    ) -> Result<Vec<Vec<u32>>> {
        let mut read_commands = Vec::with_capacity(width as usize * height as usize);
        for x in x_offset..x_offset + width as i16 {
            for y in y_offset..y_offset + height as i16 {
                if x >= 0 && x < screen_width as i16 && y >= 0 && y < screen_height as i16 {
                    read_commands.push(PixelflutRequest::GetPixel {
                        x: x as u16,
                        y: y as u16,
                    });
                }
            }
        }
        self.write_commands(&read_commands).await?;

        let mut result = vec![vec![0_u32; height as usize]; width as usize];
        let responses = self.read_commands(read_commands.len()).await?;
        for response in responses {
            match response {
                PixelflutResponse::Pixel { x, y, rgb } => {
                    result[(x as i16 - x_offset) as usize][(y as i16 - y_offset) as usize] = rgb;
                }
                _ => panic!("Expected to get the color of a pixel, but got {response:?}"),
            }
        }

        Ok(result)
    }
}
