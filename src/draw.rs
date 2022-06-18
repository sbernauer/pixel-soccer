use std::{io::Result, sync::Arc};

use async_trait::async_trait;
use tokio::task::JoinHandle;

use crate::client::Client;

#[async_trait]
pub trait Draw {
    async fn draw(&self, client: &mut Client) -> Result<()>;
}

pub async fn start_drawing(
    object: Arc<impl Draw + std::marker::Send + std::marker::Sync + 'static>,
    server_address: &str,
    num_threads: u16,
) -> Vec<JoinHandle<()>> {
    let mut threads = vec![];

    for _ in 0..num_threads {
        let mut client = Client::new(server_address).await.unwrap();
        let object_clone = object.clone();

        let thread = tokio::spawn(async move {
            loop {
                object_clone.draw(&mut client).await.unwrap();
            }
        });
        threads.push(thread);
    }

    threads
}
