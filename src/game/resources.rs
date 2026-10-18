use std::time::Duration;

use bevy::prelude::*;

use crate::core::{Cell, FoodSpawner, RandomSpawner};

use super::config::GameConfig;

/// Сегменты змейки от головы к хвосту.
#[derive(Resource, Default, Debug)]
pub struct SnakeBody(pub Vec<Entity>);

/// Клетка, которую хвост освободил на последнем ходу. Сюда встаёт новый сегмент при росте.
#[derive(Resource, Default, Debug)]
pub struct LastTail(pub Option<Cell>);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct Score(pub u32);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct BestScore(pub u32);

/// Такт движения: змейка делает ход, когда таймер срабатывает.
#[derive(Resource, Debug, Deref, DerefMut)]
pub struct MoveTimer(pub Timer);

impl MoveTimer {
    pub fn new(interval: Duration) -> Self {
        Self(Timer::new(interval, TimerMode::Repeating))
    }
}

impl FromWorld for MoveTimer {
    fn from_world(world: &mut World) -> Self {
        let speed = world.get_resource::<GameConfig>().map(|c| c.speed).unwrap_or_default();
        Self::new(speed.interval(0))
    }
}

/// Источник позиций еды. Тесты подменяют его детерминированным.
#[derive(Resource)]
pub struct FoodSource(pub Box<dyn FoodSpawner>);

impl FoodSource {
    pub fn new(spawner: impl FoodSpawner + 'static) -> Self {
        Self(Box::new(spawner))
    }
}

impl FromWorld for FoodSource {
    fn from_world(world: &mut World) -> Self {
        let seed = world.get_resource::<GameConfig>().and_then(|c| c.seed);
        Self::new(seed.map_or_else(RandomSpawner::from_entropy, RandomSpawner::seeded))
    }
}

/// Чем закончилась последняя партия; `None`, пока ни одна не закончилась.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastRound(pub Option<RoundEnd>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundEnd {
    HitWall,
    HitSelf,
    /// Змейка заняла всё поле - победа.
    BoardFilled,
}
