use std::sync::Arc;

use crate::{
    ball::{self, Ball},
    client::Client,
    draw,
    field::Field,
};
use tokio::io::Result;

pub struct Game {
    client: Client,
    field: Field,
    ball: Ball,
}

impl Game {
    pub async fn new(server_address: &str) -> Result<Self> {
        let mut client = Client::new(server_address).await?;
        let (screen_width, screen_height) = client.get_screen_size().await.unwrap();

        let ball = Ball::new(screen_width, screen_height).await?;
        let field = Field::new();

        Ok(Game {
            client,
            field,
            ball,
        })
    }

    pub async fn start(self, server_address: &str) -> Result<()> {
        let ball = Arc::new(self.ball);
        let ball_2 = Arc::clone(&ball);
        let field = Arc::new(self.field);

        let mut threads = vec![ball::start_update_thread(ball, self.client, 30)];
        threads.extend(draw::start_drawing(ball_2, server_address, 1).await);
        threads.extend(draw::start_drawing(field, server_address, 1).await);

        for thread in threads {
            thread.await?;
        }

        Ok(())
    }
}
