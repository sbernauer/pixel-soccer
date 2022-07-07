use async_trait::async_trait;
use rand::{prelude::SliceRandom, thread_rng};
use rusttype::Font;
use std::{
    io::Result,
    sync::atomic::{
        AtomicU32,
        Ordering::{AcqRel, Acquire},
    },
};
use tokio::sync::RwLock;

use crate::{
    client::{Client, AVG_BYES_PER_PIXEL_SET_COMMAND},
    draw::Draw,
    game::GoalScored,
    image_helpers::{self, BLACK, WHITE},
    protocol::Serialize,
};

pub struct Score {
    points_left: AtomicU32,
    points_right: AtomicU32,

    font: Font<'static>,

    draw_command_bytes: RwLock<Vec<u8>>,
}

impl Score {
    pub async fn new() -> Self {
        let font = Font::try_from_bytes(include_bytes!("../Arial.ttf"))
            .unwrap_or_else(|| panic!("Failed to construct Font from Arial.ttf"));
        let score = Score {
            points_left: AtomicU32::new(0),
            points_right: AtomicU32::new(0),
            font,
            draw_command_bytes: RwLock::new(vec![]),
        };
        score.update_draw_commands().await;
        score
    }

    pub async fn score_goal(&self, goal: GoalScored) {
        match goal {
            GoalScored::Left => {
                self.points_right.fetch_add(1, AcqRel);
            }
            GoalScored::Right => {
                self.points_left.fetch_add(1, AcqRel);
            }
        }
        self.update_draw_commands().await;
    }

    async fn update_draw_commands(&self) {
        let mut draw_commands = image_helpers::draw_text_with_background(
            20,
            300,
            100,
            54,
            60.0,
            BLACK,
            WHITE,
            self.points_left.load(Acquire).to_string().as_str(),
            &self.font,
        );
        draw_commands.extend(image_helpers::draw_text_with_background(
            1798,
            300,
            100,
            54,
            60.0,
            BLACK,
            WHITE,
            self.points_right.load(Acquire).to_string().as_str(),
            &self.font,
        ));

        // Shuffle commands to prevent drawing artefacts
        draw_commands.shuffle(&mut thread_rng());

        let mut draw_command_bytes =
            Vec::with_capacity(draw_commands.len() * AVG_BYES_PER_PIXEL_SET_COMMAND);
        draw_commands.iter().for_each(|cmd| {
            cmd.serialize(&mut draw_command_bytes);
        });

        *(self.draw_command_bytes.write().await) = draw_command_bytes;
    }
}

#[async_trait]
impl Draw for Score {
    async fn draw(&self, client: &mut Client) -> Result<()> {
        client
            .write_bytes(self.draw_command_bytes.read().await.as_ref())
            .await?;
        Ok(())
    }
}
