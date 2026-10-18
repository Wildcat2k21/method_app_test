//! Общий стенд для тестов лабораторных работ.
//!
//! [`Harness`] - игра без окна и рендера (`MinimalPlugins` + логика). Тест управляет ею
//! как игрок: нажимает клавиши (через те же сообщения `KeyboardInput`, что шлёт окно)
//! и проматывает время ровно на один ход змейки. Еду можно задать сценарием
//! ([`ScriptedSpawner`]), чтобы партия была воспроизводимой.
//!
//! Подключение в тестовой цели: `#[path = "../common/mod.rs"] mod common;`

#![allow(dead_code)] // каждая работа использует свою часть стенда

use std::collections::VecDeque;
use std::time::Duration;

use bevy::input::ButtonState;
use bevy::input::InputPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use snake_game::core::{Board, Cell, Direction, FoodSpawner, SpeedRules};
use snake_game::game::*;

/// Еда по списку: берёт очередную клетку, если она свободна, иначе первую свободную.
/// Список кончился - тоже первая свободная. Так партия полностью детерминирована.
#[derive(Default)]
pub struct ScriptedSpawner {
    queue: VecDeque<Cell>,
}

impl ScriptedSpawner {
    pub fn new(cells: impl IntoIterator<Item = Cell>) -> Self {
        Self { queue: cells.into_iter().collect() }
    }
}

impl FoodSpawner for ScriptedSpawner {
    fn pick(&mut self, free: &[Cell]) -> Option<Cell> {
        match self.queue.pop_front() {
            Some(cell) if free.contains(&cell) => Some(cell),
            _ => free.first().copied(),
        }
    }
}

/// Конфигурация для тестов: поле `width × height`, змейка из 3 клеток вправо,
/// скорость по умолчанию, без файла рекорда.
pub fn config(width: i32, height: i32) -> GameConfig {
    GameConfig {
        board: Board::new(width, height),
        initial_len: 3,
        start_direction: Direction::Right,
        speed: SpeedRules::default(),
        seed: Some(1),
        best_score_file: None,
    }
}

pub struct Harness {
    pub app: App,
}

impl Harness {
    /// Игра на экране меню. Еда - по `food`, затем первая свободная клетка.
    pub fn new(cfg: GameConfig, food: impl IntoIterator<Item = Cell>) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, InputPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .insert_resource(cfg)
            .add_plugins(SnakeLogicPlugin)
            .insert_resource(FoodSource::new(ScriptedSpawner::new(food)));
        app.update(); // Startup и OnEnter(Menu)
        Self { app }
    }

    /// Сразу в партии: меню пройдено, змейка и еда на поле.
    pub fn playing(cfg: GameConfig, food: impl IntoIterator<Item = Cell>) -> Self {
        let mut h = Self::new(cfg, food);
        h.start();
        h
    }

    // ---------- Действия игрока ----------

    /// Нажатие и отпускание клавиши. Время не идёт.
    pub fn press(&mut self, key: KeyCode) {
        self.send_key(key, ButtonState::Pressed);
        self.app.update();
        self.send_key(key, ButtonState::Released);
    }

    /// Enter в меню и кадр на переход в `Playing`.
    pub fn start(&mut self) {
        self.press(KeyCode::Enter);
        self.frame();
    }

    pub fn turn(&mut self, dir: Direction) {
        self.press(match dir {
            Direction::Up => KeyCode::ArrowUp,
            Direction::Down => KeyCode::ArrowDown,
            Direction::Left => KeyCode::ArrowLeft,
            Direction::Right => KeyCode::ArrowRight,
        });
    }

    /// Кадр без хода времени.
    pub fn frame(&mut self) {
        self.app.update();
    }

    /// Кадр, в котором время продвигается на `dt`.
    pub fn advance(&mut self, dt: Duration) {
        self.app.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
        self.app.update();
        self.app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    }

    /// Ровно один ход змейки: время до срабатывания таймера хода.
    pub fn tick(&mut self) {
        let remaining = self.app.world().resource::<MoveTimer>().remaining();
        self.advance(remaining);
    }

    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.tick();
        }
    }

    // ---------- Наблюдение ----------

    pub fn state(&self) -> GameState {
        *self.app.world().resource::<State<GameState>>().get()
    }

    /// `None` вне `Playing`.
    pub fn pause(&self) -> Option<Pause> {
        self.app.world().get_resource::<State<Pause>>().map(|s| *s.get())
    }

    /// Клетки змейки от головы к хвосту.
    pub fn snake(&mut self) -> Vec<Cell> {
        let body = self.app.world().resource::<SnakeBody>().0.clone();
        let world = self.app.world_mut();
        let mut positions = world.query::<&GridPos>();
        body.iter().map(|&e| positions.get(world, e).expect("сегмент без GridPos").0).collect()
    }

    pub fn head(&mut self) -> Cell {
        self.snake()[0]
    }

    pub fn food(&mut self) -> Option<Cell> {
        let world = self.app.world_mut();
        let mut food = world.query_filtered::<&GridPos, With<Food>>();
        let all: Vec<Cell> = food.iter(world).map(|p| p.0).collect();
        assert!(all.len() <= 1, "на поле несколько единиц еды: {all:?}");
        all.first().copied()
    }

    pub fn heading(&mut self) -> Heading {
        let world = self.app.world_mut();
        *world.query_filtered::<&Heading, With<SnakeHead>>().single(world).expect("одна голова")
    }

    pub fn score(&self) -> u32 {
        self.app.world().resource::<Score>().0
    }

    pub fn best(&self) -> u32 {
        self.app.world().resource::<BestScore>().0
    }

    pub fn last_round(&self) -> Option<RoundEnd> {
        self.app.world().resource::<LastRound>().0
    }

    pub fn interval(&self) -> Duration {
        self.app.world().resource::<MoveTimer>().duration()
    }

    fn send_key(&mut self, key_code: KeyCode, state: ButtonState) {
        self.app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }
}
