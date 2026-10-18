//! ЛР5. Модульные тесты ECS-систем (`crate::game`).
//!
//! Подключён через `#[path]` в src/game/mod.rs как дочерний модуль `crate::game`,
//! поэтому видит системы `logic::*`, закрытые для внешнего кода (`pub(super)`).
//!
//! Единица тестирования - одна система. Вместо приложения с расписанием -
//! пустой `World` с минимумом ресурсов и один вызов `run_system_once`.
//! Зависимость от случайности заменена тестовым двойником (`SpySpawner`).

use std::sync::{Arc, Mutex};

use bevy::ecs::message::Messages;
use bevy::ecs::system::RunSystemOnce;

use super::logic::{eat_food, move_snake, spawn_food};
use super::*;
use crate::core::{Board, Cell, Direction, FoodSpawner};

// ---------- Heading (компонент с поведением) ----------

#[test]
fn heading_rejects_reverse_even_after_queued_turn() {
    let mut h = Heading::new(Direction::Right);

    assert!(h.request(Direction::Up));
    assert!(!h.request(Direction::Left), "Left - разворот относительно current = Right");

    assert_eq!(h.queued, Some(Direction::Up));
}

#[test]
fn heading_advance_consumes_queue_once() {
    let mut h = Heading::new(Direction::Right);
    h.request(Direction::Down);

    assert_eq!(h.advance(), Direction::Down);
    assert_eq!(h.advance(), Direction::Down);
    assert_eq!(h.queued, None);
}

// ---------- Мир для одной системы ----------

fn world_with_snake(board: Board, cells: &[Cell], dir: Direction) -> World {
    let mut world = World::new();
    world.insert_resource(GameConfig { board, ..default() });
    world.init_resource::<LastTail>();
    world.init_resource::<Score>();
    world.init_resource::<Messages<RoundOver>>();
    world.init_resource::<Messages<FoodEaten>>();
    let body = cells
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let mut e = world.spawn((SnakeSegment, GridPos(c)));
            if i == 0 {
                e.insert((SnakeHead, Heading::new(dir)));
            }
            e.id()
        })
        .collect();
    world.insert_resource(SnakeBody(body));
    world
}

fn snake_cells(world: &World) -> Vec<Cell> {
    world.resource::<SnakeBody>().0.iter().map(|&e| world.get::<GridPos>(e).unwrap().0).collect()
}

fn round_over(world: &World) -> Vec<RoundOver> {
    world.resource::<Messages<RoundOver>>().iter_current_update_messages().copied().collect()
}

// ---------- move_snake ----------

#[test]
fn move_snake_shifts_every_segment() {
    let start = [Cell::new(2, 2), Cell::new(1, 2), Cell::new(0, 2)];
    let mut world = world_with_snake(Board::new(5, 5), &start, Direction::Right);

    world.run_system_once(move_snake).unwrap();

    assert_eq!(snake_cells(&world), vec![Cell::new(3, 2), Cell::new(2, 2), Cell::new(1, 2)]);
    assert_eq!(world.resource::<LastTail>().0, Some(Cell::new(0, 2)));
    assert!(round_over(&world).is_empty());
}

#[test]
fn move_snake_reports_wall_and_stays() {
    let start = [Cell::new(4, 2), Cell::new(3, 2)];
    let mut world = world_with_snake(Board::new(5, 5), &start, Direction::Right);

    world.run_system_once(move_snake).unwrap();

    assert_eq!(round_over(&world), vec![RoundOver(RoundEnd::HitWall)]);
    assert_eq!(snake_cells(&world), start.to_vec(), "после столкновения змейка не двигается");
}

// ---------- eat_food ----------

#[test]
fn eat_food_scores_and_despawns() {
    let mut world = world_with_snake(Board::new(5, 5), &[Cell::new(2, 2)], Direction::Right);
    let food = world.spawn((Food, GridPos(Cell::new(2, 2)))).id();

    world.run_system_once(eat_food).unwrap();

    assert_eq!(world.resource::<Score>().0, 1);
    assert!(world.get_entity(food).is_err(), "еда удалена");
    let eaten: Vec<_> = world.resource::<Messages<FoodEaten>>().iter_current_update_messages().copied().collect();
    assert_eq!(eaten, vec![FoodEaten { at: Cell::new(2, 2) }]);
}

#[test]
fn eat_food_ignores_food_elsewhere() {
    let mut world = world_with_snake(Board::new(5, 5), &[Cell::new(2, 2)], Direction::Right);
    world.spawn((Food, GridPos(Cell::new(4, 4))));

    world.run_system_once(eat_food).unwrap();

    assert_eq!(world.resource::<Score>().0, 0);
}

// ---------- spawn_food с тестовым двойником ----------

/// Шпион: запоминает, какие свободные клетки ему показали, и отвечает заданной клеткой.
struct SpySpawner {
    seen: Arc<Mutex<Vec<Cell>>>,
    answer: Option<Cell>,
}

impl FoodSpawner for SpySpawner {
    fn pick(&mut self, free: &[Cell]) -> Option<Cell> {
        *self.seen.lock().unwrap() = free.to_vec();
        self.answer
    }
}

#[test]
fn spawn_food_offers_only_free_cells_and_places_answer() {
    let mut world = world_with_snake(Board::new(2, 2), &[Cell::new(0, 0), Cell::new(1, 0)], Direction::Up);
    let seen = Arc::new(Mutex::new(Vec::new()));
    world.insert_resource(FoodSource::new(SpySpawner { seen: seen.clone(), answer: Some(Cell::new(1, 1)) }));

    world.run_system_once(spawn_food).unwrap();

    assert_eq!(*seen.lock().unwrap(), vec![Cell::new(0, 1), Cell::new(1, 1)]);
    let mut food = world.query_filtered::<&GridPos, With<Food>>();
    assert_eq!(food.iter(&world).map(|p| p.0).collect::<Vec<_>>(), vec![Cell::new(1, 1)]);
}

#[test]
fn spawn_food_without_place_wins_round() {
    let mut world = world_with_snake(Board::new(2, 2), &[Cell::new(0, 0)], Direction::Up);
    world.insert_resource(FoodSource::new(SpySpawner { seen: default(), answer: None }));

    world.run_system_once(spawn_food).unwrap();

    assert_eq!(round_over(&world), vec![RoundOver(RoundEnd::BoardFilled)]);
}
