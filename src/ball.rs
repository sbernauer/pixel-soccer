use async_trait::async_trait;
use atomic_float::AtomicF32;
use image::{io::Reader as ImageReader, DynamicImage};
use std::{
    f32::consts::PI,
    io::Result,
    sync::{
        atomic::Ordering::{Acquire, Release},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::RwLock, task::JoinHandle, time};

use crate::{
    client::{Client, AVG_BYES_PER_PIXEL_SET_COMMAND},
    draw::Draw,
    image_helpers,
    protocol::Serialize,
};

const SPEED: f32 = 10.0_f32;
const LOOKAHEAD_LINE_LENGTH: f32 = 100_f32; // Keep in mind that for a distance of BALL_IMAGE_DIAMETER / 2 nothing will be looked at because it's inside the ball.

const TARGET_COLOR: u32 = 0x00ff0000; // red
const TARGET_COLOR_THRESHOLD: f32 = 0.99f32;

// Measure the following variables with an Image editing program
const BALL_IMAGE_SIZE: u16 = 80; // Assuming quadratic image the width and height of the image
const BALL_RADIUS: f32 = 40_f32;

pub struct Ball {
    image: DynamicImage,
    draw_command_bytes: RwLock<Vec<u8>>,

    center_x: AtomicF32,
    center_y: AtomicF32,
    dir: AtomicF32,
    speed: f32,

    screen_width: u16,
    screen_height: u16,
}

impl Ball {
    pub async fn new(screen_width: u16, screen_height: u16) -> Result<Self> {
        let image = ImageReader::open("images/ball_v1.png")?
            .decode()
            .expect("Failed to decode ball image");

        let ball = Ball {
            image,
            draw_command_bytes: RwLock::new(vec![]),
            center_x: AtomicF32::new(((screen_width - BALL_IMAGE_SIZE) / 2) as f32),
            center_y: AtomicF32::new(((screen_height - BALL_IMAGE_SIZE) / 2) as f32),
            dir: AtomicF32::new(-PI / 0.9_f32),
            speed: SPEED,
            screen_width,
            screen_height,
        };
        ball.update_draw_command_bytes().await;

        Ok(ball)
    }

    async fn update_draw_command_bytes(&self) {
        let draw_commands = image_helpers::draw_image(
            &self.image,
            (self.center_x.load(Acquire) - BALL_RADIUS) as u16,
            (self.center_y.load(Acquire) - BALL_RADIUS) as u16,
        );
        let mut draw_command_bytes =
            Vec::with_capacity(draw_commands.len() * AVG_BYES_PER_PIXEL_SET_COMMAND);
        draw_commands.iter().for_each(|cmd| {
            cmd.serialize(&mut draw_command_bytes);
        });

        *(self.draw_command_bytes.write().await) = draw_command_bytes;
    }

    async fn tick(&self) {
        let mut new_dir = self.dir.load(Acquire);
        let mut new_center_x = self.center_x.load(Acquire);
        let mut new_center_y = self.center_y.load(Acquire);
        let mut movement_x = self.speed * new_dir.cos();
        let mut movement_y = self.speed * new_dir.sin();

        new_center_x += movement_x;
        new_center_y += movement_y;

        // Collision on left or right
        if new_center_x - BALL_RADIUS <= 0_f32
            || new_center_x + BALL_RADIUS >= self.screen_width as f32
        {
            movement_x *= -1_f32;
        }

        // Collision on top or bottom
        if new_center_y - BALL_RADIUS <= 0_f32
            || new_center_y + BALL_RADIUS >= self.screen_height as f32
        {
            movement_y *= -1_f32;
        }

        new_dir = movement_y.atan2(movement_x);

        self.center_x.store(new_center_x, Release);
        self.center_y.store(new_center_y, Release);
        self.dir.store(new_dir, Release);

        self.update_draw_command_bytes().await;
    }
}

#[async_trait]
impl Draw for Ball {
    async fn draw(&self, client: &mut Client) -> Result<()> {
        client
            .write_bytes(self.draw_command_bytes.read().await.as_ref())
            .await?;
        Ok(())
    }
}

pub fn start_update_thread(ball: Arc<Ball>, target_fps: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(1_000 / target_fps));
        loop {
            ball.tick().await;
            interval.tick().await;
        }
    })
}

pub fn start_draw_thread(ball: Arc<Ball>, mut client: Client) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        loop {
            ball.draw(&mut client).await?;
        }
    })
}
