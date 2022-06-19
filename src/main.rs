use clap::Parser;
use client::Client;
use std::{io::Result, sync::Arc};

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

    let ball = Arc::new(Ball::new(screen_width, screen_height).await?);

    let mut threads = vec![ball::start_update_thread(Arc::clone(&ball), client, 30)];
    threads.extend(draw::start_drawing(ball, &args.server_address, 1).await);

    for thread in threads {
        thread.await?;
    }
    Ok(())
}
