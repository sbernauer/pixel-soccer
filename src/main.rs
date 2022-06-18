use client::Client;
use std::io::Result;

mod client;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("127.0.0.1:1234").await?;

    let (screen_width, screen_height) = client.get_screen_size().await.unwrap();
    dbg!(screen_width);
    dbg!(screen_height);

    Ok(())
}
