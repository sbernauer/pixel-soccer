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
const LOOK_RADIUS: u16 = 46; // ca. BALL_RADIUS + SPEED / 2.0 + 1

const TARGET_COLOR: u32 = 0x00ff0000; // red

// Measure the following variables with an Image editing program
const BALL_IMAGE_SIZE: u16 = 80; // Assuming quadratic image the width and height of the image
const BALL_RADIUS: f32 = 40_f32;

pub struct Ball {
    image: DynamicImage,
    draw_command_bytes: RwLock<Vec<u8>>,

    center_x: AtomicF32,
    center_y: AtomicF32,
    dir: AtomicF32,

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

    async fn tick(&self, client: &mut Client) -> Result<()> {
        let dir = self.dir.load(Acquire);
        let center_x = self.center_x.load(Acquire);
        let center_y = self.center_y.load(Acquire);
        let mut movement_x = SPEED * self.dir.load(Acquire).cos();
        let mut movement_y = SPEED * self.dir.load(Acquire).sin();

        let mut bounced_with_edge = false;
        // Collision on left or right
        if center_x - BALL_RADIUS <= 0_f32 || center_x + BALL_RADIUS >= self.screen_width as f32 {
            movement_x *= -1_f32;
            bounced_with_edge = true;
        }

        // Collision on top or bottom
        if center_y - BALL_RADIUS <= 0_f32 || center_y + BALL_RADIUS >= self.screen_height as f32 {
            movement_y *= -1_f32;
            bounced_with_edge = true;
        }

        // Ask for a rect with the ball in the center
        let rect = client
            .get_screen_rect(
                center_x as i16 - LOOK_RADIUS as i16,
                center_y as i16 - LOOK_RADIUS as i16,
                2 * LOOK_RADIUS,
                2 * LOOK_RADIUS,
                self.screen_width,
                self.screen_height,
            )
            .await?;

        let mut contains_red = false;
        let mut min_x_value = 0.0;
        let mut min_y_value = 0.0;
        let mut min_distance = f32::MAX;

        #[allow(clippy::needless_range_loop)]
        for x in 0..2 * LOOK_RADIUS as usize {
            for y in 0..2 * LOOK_RADIUS as usize {
                if rect[x][y] == TARGET_COLOR {
                    contains_red = true;
                    let x_rel = x as f32 - LOOK_RADIUS as f32;
                    let y_rel = y as f32 - LOOK_RADIUS as f32;
                    let distance = f32::sqrt(f32::powi(x_rel, 2) + f32::powi(y_rel, 2));
                    if distance < min_distance {
                        min_distance = distance;
                        min_x_value = x_rel;
                        min_y_value = y_rel;
                    }
                }
            }
        }

        if !bounced_with_edge
            && contains_red
            && min_distance <= BALL_RADIUS + SPEED / 2.0
            && min_distance >= BALL_RADIUS - SPEED / 2.0
        {
            // Calculate direction to nearest red point
            let nearest_red_dir = min_y_value.atan2(min_x_value);
            let nearest_red_dir_reflect_vector = nearest_red_dir + PI;

            // And the new direction to go after bounce
            let bounce_dir =
                nearest_red_dir_reflect_vector - (dir + PI - nearest_red_dir_reflect_vector);

            movement_x = SPEED * bounce_dir.cos();
            movement_y = SPEED * bounce_dir.sin();

            // println!("BOUNCE: dir {dir} nearest_red_dir_reflect_vector: {nearest_red_dir_reflect_vector} bounce_dir {bounce_dir}");
        }

        self.center_x.store(center_x + movement_x, Release);
        self.center_y.store(center_y + movement_y, Release);
        self.dir.store(movement_y.atan2(movement_x), Release);

        self.update_draw_command_bytes().await;

        Ok(())
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

pub fn start_update_thread(ball: Arc<Ball>, mut client: Client, target_fps: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(1_000 / target_fps));
        loop {
            // let start = Instant::now();
            ball.tick(&mut client).await.unwrap();
            // println!("Took {:?} to tick the ball", start.elapsed());
            interval.tick().await;
        }
    })
}
