use bevy::prelude::*;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

/// Пауза - подсостояние `Playing`: переключение паузы не вызывает
/// `OnEnter(GameState::Playing)` и не начинает партию заново.
#[derive(SubStates, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum Pause {
    #[default]
    Running,
    Paused,
}
