use client::Client;
use std::io::Result;

use crate::protocol::PixelflutRequest;

mod client;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("127.0.0.1:1234").await?;

    let (screen_width, screen_height) = client.get_screen_size().await.unwrap();
    dbg!(screen_width);
    dbg!(screen_height);

    client
        .write_commands(&vec![
            PixelflutRequest::SetPixel {
                x: 0,
                y: 0,
                rgb: 0xabcdef,
            },
            PixelflutRequest::GetPixel { x: 0, y: 0 },
        ])
        .await?;
    let get_pixel_response = client.read_commands(1).await?;
    dbg!(get_pixel_response);

    Ok(())
}
