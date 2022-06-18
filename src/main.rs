use client::Client;
use std::{io::Result, sync::Arc};

use crate::ball::Ball;

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

    let ball = Arc::new(Ball::new(screen_width, screen_height).await?);
    let ball_update_thread = ball::start_update_thread(Arc::clone(&ball), 30);
    let ball_draw_thread =
        ball::start_draw_thread(Arc::clone(&ball), Client::new("127.0.0.1:1234").await?);

    ball_update_thread.await?;
    ball_draw_thread.await??;
    Ok(())
}
