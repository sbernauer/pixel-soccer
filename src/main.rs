use clap::Parser;
use client::Client;
use std::{io::Result, sync::Arc};
use tokio::time::Instant;

use crate::{args::Args, ball::Ball};

mod args;
mod ball;
mod client;
mod draw;
mod image_helpers;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut client = Client::new(&args.server_address).await?;
    let (screen_width, screen_height) = client.get_screen_size().await.unwrap();

    for _ in 0..5 {
        let start = Instant::now();
        client
            .get_screen_rect(0, 0, 100, 100, screen_width, screen_height)
            .await?;
        dbg!(start.elapsed());
    }

    let ball = Arc::new(Ball::new(screen_width, screen_height).await?);

    let mut threads = vec![ball::start_update_thread(Arc::clone(&ball), 30)];
    threads.extend(draw::start_drawing(ball, &args.server_address, 1).await);

    for thread in threads {
        thread.await?;
    }
    Ok(())
}
