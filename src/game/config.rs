use std::path::PathBuf;

use bevy::prelude::*;

use crate::core::{Board, Direction, SpeedRules};

/// Параметры партии. Вставляется в `App` до [`super::SnakeLogicPlugin`];
/// если её нет, плагин берёт `GameConfig::default()`.
#[derive(Resource, Clone, Debug)]
pub struct GameConfig {
    pub board: Board,
    pub initial_len: usize,
    pub start_direction: Direction,
    pub speed: SpeedRules,
    /// `Some` - воспроизводимая последовательность еды, `None` - случайная.
    pub seed: Option<u64>,
    /// Файл рекорда; `None` - рекорд живёт только до закрытия приложения.
    pub best_score_file: Option<PathBuf>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board: Board::new(20, 15),
            initial_len: 3,
            start_direction: Direction::Right,
            speed: SpeedRules::default(),
            seed: None,
            best_score_file: None,
        }
    }
}
