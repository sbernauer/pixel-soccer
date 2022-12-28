use std::{sync::Arc, time::Duration};

use crate::{ball::Ball, client::Client, draw, field::Field, score::Score};
use tokio::{
    io::Result,
    time::{self, Instant},
};

pub struct Game {
    client: Client,
    field: Field,
    ball: Ball,
    score: Score,
}

pub enum GoalScored {
    Left,
    Right,
}

impl Game {
    pub async fn new(server_address: &str) -> Result<Self> {
        let mut client = Client::new(server_address).await?;
        let (screen_width, screen_height) = client.get_screen_size().await.unwrap();
        println!("Detected screen size {screen_width} x {screen_height}");

        let ball = Ball::new(screen_width, screen_height).await?;

        Ok(Game {
            client,
            field: Field::new(),
            ball,
            score: Score::new().await,
        })
    }

    pub async fn start(mut self, server_address: &str, target_fps: u16) -> Result<()> {
        let ball = Arc::new(self.ball);
        let ball_2 = Arc::clone(&ball);
        let field = Arc::new(self.field);
        let score = Arc::new(self.score);
        let score_2 = Arc::clone(&score);

        let mut threads = Vec::new();

        let mut fps_counter_last_update = Instant::now();
        let mut fps_counter = 0;
        let mut interval = time::interval(Duration::from_millis(1_000 / target_fps as u64));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        threads.push(tokio::spawn(async move {
            loop {
                interval.tick().await;

                // let start = Instant::now();
                ball.tick(&mut self.client).await.unwrap();
                // println!("Took {:?} to tick the ball", start.elapsed());

                if let Some(goal) = ball.is_goal_scored() {
                    score.score_goal(goal).await;
                    ball.reset();
                }

                if fps_counter_last_update.elapsed() >= Duration::from_secs(1) {
                    println!("{} fps", fps_counter);
                    fps_counter = 0;
                    fps_counter_last_update = Instant::now();
                } else {
                    fps_counter += 1;
                }
            }
        }));

        threads.extend(draw::start_drawing(ball_2, server_address, 1).await);
        threads.extend(draw::start_drawing(field, server_address, 3).await);
        threads.extend(draw::start_drawing(score_2, server_address, 1).await);

        for thread in threads {
            thread.await?;
        }

        Ok(())
    }
}
