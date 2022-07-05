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
use tokio::{
    sync::RwLock,
    task::JoinHandle,
    time::{self, Instant},
};

use crate::{
    client::{Client, AVG_BYES_PER_PIXEL_SET_COMMAND},
    draw::Draw,
    image_helpers,
    protocol::Serialize,
};

pub const TARGET_COLOR: u32 = 0x00ff_0000; // red

// ####################
// Be careful about changing any of these constants below!
// They can change the way the ball behaves, in the worst case letting it glitch through walls or bounce in the wrong direction
// ####################
const SPEED: f32 = 10.0_f32;
// Measure the following variables with an image editing program
const BALL_IMAGE_SIZE: u16 = 80; // Assuming quadratic image this is the width and height of the image
const BALL_RADIUS: f32 = 40_f32;

pub struct Ball {
    image: DynamicImage,
    draw_command_bytes: RwLock<Vec<u8>>,

    field_hitbox_image: DynamicImage,

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

        let field_hitbox_image = ImageReader::open("images/field_v3_hitbox.png")?
            .decode()
            .expect("Failed to decode field hitbox image");

        let ball = Ball {
            image,
            draw_command_bytes: RwLock::new(vec![]),
            field_hitbox_image,
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

        let inner_circle_radius = BALL_RADIUS - SPEED / 2.0;
        let outer_circle_radius = BALL_RADIUS + SPEED / 2.0;

        let donut = client
            .get_screen_donut(
                center_x as i16,
                center_y as i16,
                inner_circle_radius,
                outer_circle_radius,
                self.screen_width,
                self.screen_height,
                Some(&self.field_hitbox_image),
            )
            .await?;

        let mut contains_red = false;
        let mut min_x_value = 0.0;
        let mut min_y_value = 0.0;
        let mut min_distance = f32::MAX;

        for x in 0..donut.len() {
            for y in 0..donut[0].len() {
                if donut[x][y] == TARGET_COLOR {
                    contains_red = true;
                    let x_rel = x as f32 - outer_circle_radius;
                    let y_rel = y as f32 - outer_circle_radius;
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
    let mut fps_counter_last_update = Instant::now();
    let mut fps_counter = 0;

    let mut interval = time::interval(Duration::from_millis(1_000 / target_fps));
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    tokio::spawn(async move {
        loop {
            interval.tick().await;

            // let start = Instant::now();
            ball.tick(&mut client).await.unwrap();
            // println!("Took {:?} to tick the ball", start.elapsed());

            if fps_counter_last_update.elapsed() >= Duration::from_secs(1) {
                println!("{} fps", fps_counter);
                fps_counter = 0;
                fps_counter_last_update = Instant::now();
            } else {
                fps_counter += 1;
            }
        }
    })
}
