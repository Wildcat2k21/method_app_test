//! ECS-слой: состояние в компонентах и ресурсах, поведение в системах.
//!
//! - [`SnakeLogicPlugin`] - правила, ввод, переходы между экранами. Не требует окна
//!   и рендера: тесты запускают его на `MinimalPlugins`.
//! - [`view::ViewPlugin`] - камера, спрайты поля, змейки и еды.
//! - [`ui::UiPlugin`] - HUD, меню, пауза, экран конца партии.
//! - [`SnakeGamePlugin`] - всё вместе, для `main`.

pub mod components;
pub mod config;
pub mod logic;
pub mod messages;
pub mod resources;
pub mod state;
pub mod ui;
pub mod view;

use bevy::prelude::*;

pub use components::*;
pub use config::GameConfig;
pub use messages::{FoodEaten, RoundOver};
pub use resources::*;
pub use state::{GameState, Pause};

use logic::*;

// Модульные тесты ЛР5 лежат в labs/lb5, но компилируются внутри модуля: `cargo test --lib lb5`.
#[cfg(test)]
#[path = "../../labs/lb5/system_units.rs"]
mod lb5;

/// Порядок кадра. Наборы `Input`–`Outcome` работают только во время игры без паузы.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SnakeSet {
    /// Переходы между экранами с клавиатуры (работает на всех экранах).
    Controls,
    /// Повороты змейки.
    Input,
    /// Ход по таймеру: движение, еда, рост, скорость.
    Tick,
    /// Новая еда, если её нет.
    Food,
    /// Конец партии.
    Outcome,
}

pub struct SnakeLogicPlugin;

impl Plugin for SnakeLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .init_state::<GameState>()
            .add_sub_state::<Pause>()
            .add_message::<FoodEaten>()
            .add_message::<RoundOver>()
            .init_resource::<SnakeBody>()
            .init_resource::<LastTail>()
            .init_resource::<Score>()
            .init_resource::<BestScore>()
            .init_resource::<LastRound>()
            .init_resource::<MoveTimer>()
            .init_resource::<FoodSource>()
            .configure_sets(
                Update,
                (SnakeSet::Controls, SnakeSet::Input, SnakeSet::Tick, SnakeSet::Food, SnakeSet::Outcome).chain(),
            )
            .configure_sets(
                Update,
                (SnakeSet::Input, SnakeSet::Tick, SnakeSet::Food, SnakeSet::Outcome).run_if(in_state(Pause::Running)),
            )
            .add_systems(Startup, load_best_score)
            .add_systems(OnEnter(GameState::Menu), clear_round)
            .add_systems(OnEnter(GameState::Playing), start_round)
            .add_systems(
                Update,
                (
                    (
                        menu_keys.run_if(in_state(GameState::Menu)),
                        pause_keys.run_if(in_state(GameState::Playing)),
                        game_over_keys.run_if(in_state(GameState::GameOver)),
                    )
                        .in_set(SnakeSet::Controls),
                    steer.in_set(SnakeSet::Input),
                    (advance_timer, (move_snake, eat_food, grow_snake, update_speed).chain().run_if(move_due))
                        .chain()
                        .in_set(SnakeSet::Tick),
                    spawn_food.in_set(SnakeSet::Food),
                    finish_round.in_set(SnakeSet::Outcome),
                ),
            );
    }
}

/// Полная игра: логика, графика и интерфейс.
pub struct SnakeGamePlugin;

impl Plugin for SnakeGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SnakeLogicPlugin, view::ViewPlugin, ui::UiPlugin));
    }
}
