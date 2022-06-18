use client::Client;
use std::io::Result;

use crate::{ball::Ball, draw::Draw};

mod ball;
mod client;
mod draw;
mod image_helpers;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("127.0.0.1:1234").await?;

    let (screen_width, screen_height) = client.get_screen_size().await.unwrap();
    dbg!(screen_width);
    dbg!(screen_height);

    let ball = Ball::new(screen_width, screen_height).await?;
    ball.draw(&mut client).await?;

    Ok(())
}
