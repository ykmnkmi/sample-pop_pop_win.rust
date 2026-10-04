pub mod animation;
pub mod assets;
pub mod audio;
pub mod board;
pub mod game;
pub mod input;
pub mod storage;
pub mod ui;

use bevy::prelude::*;
use game::Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}

impl GameDifficulty {
    pub fn dimensions(&self) -> (usize, usize, usize) {
        match self {
            GameDifficulty::Easy => (7, 7, 7),
            GameDifficulty::Medium => (11, 11, 18),
            GameDifficulty::Hard => (16, 16, 40),
            GameDifficulty::Extreme => (24, 24, 90),
        }
    }
}

#[derive(Resource)]
pub struct GameWrapper(pub Game);
