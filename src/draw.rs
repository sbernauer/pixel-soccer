use std::io::Result;

use async_trait::async_trait;

use crate::client::Client;

#[async_trait]
pub trait Draw {
    async fn draw(&self, client: &mut Client) -> Result<()>;
}
