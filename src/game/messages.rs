use bevy::prelude::*;

use crate::core::Cell;

use super::resources::RoundEnd;

/// Голова съела еду в клетке `at`.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoodEaten {
    pub at: Cell,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundOver(pub RoundEnd);
