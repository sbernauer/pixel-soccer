use crate::args::Args;
use clap::Parser;
use game::Game;
use tokio::io::Result;

mod args;
mod ball;
mod client;
mod draw;
mod field;
mod game;
mod image_helpers;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let game = Game::new(&args.server_address).await?;
    game.start(&args.server_address).await?;

    Ok(())
}
