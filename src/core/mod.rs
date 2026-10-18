//! Правила змейки без зависимостей от Bevy.
//!
//! ECS-слой ([`crate::game`]) хранит состояние в компонентах и ресурсах, а решения
//! «можно ли повернуть», «во что врезалась голова», «где появится еда», «какая сейчас
//! скорость» принимает здесь, чистыми функциями. Их можно проверять без движка.

pub mod food;
pub mod grid;
pub mod highscore;
pub mod rules;

pub use food::{FoodSpawner, RandomSpawner};
pub use grid::{Board, Cell, Direction};
pub use rules::{Collision, SpeedRules};

// Модульные тесты ЛР5 лежат в labs/lb5, но компилируются внутри модуля: `cargo test --lib lb5`.
#[cfg(test)]
#[path = "../../labs/lb5/core_units.rs"]
mod lb5;
