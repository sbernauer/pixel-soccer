use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Address to connect to.
    #[clap(short, long, default_value = "[::1]:1234")]
    pub server_address: String,

    /// Frames per second the game should try to reach.
    /// This is mainly limited by the (network) latency to read the pixel values.
    #[clap(short, long, default_value = "20")]
    pub fps: u16,
}
