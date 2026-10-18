//! Системы игровой логики. Работают без окна и рендера.

use std::iter;

use bevy::prelude::*;

use crate::core::{Cell, Collision, Direction, highscore, rules};

use super::components::*;
use super::config::GameConfig;
use super::messages::{FoodEaten, RoundOver};
use super::resources::*;
use super::state::{GameState, Pause};

/// Стрелки и WASD.
pub fn key_direction(key: KeyCode) -> Option<Direction> {
    match key {
        KeyCode::ArrowUp | KeyCode::KeyW => Some(Direction::Up),
        KeyCode::ArrowDown | KeyCode::KeyS => Some(Direction::Down),
        KeyCode::ArrowLeft | KeyCode::KeyA => Some(Direction::Left),
        KeyCode::ArrowRight | KeyCode::KeyD => Some(Direction::Right),
        _ => None,
    }
}

// ---------- Начало и конец партии ----------

pub(super) fn load_best_score(cfg: Res<GameConfig>, mut best: ResMut<BestScore>) {
    if let Some(path) = &cfg.best_score_file {
        best.0 = highscore::load(path);
    }
}

pub(super) fn clear_round(
    mut commands: Commands,
    entities: Query<Entity, Or<(With<SnakeSegment>, With<Food>)>>,
    mut body: ResMut<SnakeBody>,
) {
    for e in &entities {
        commands.entity(e).despawn();
    }
    body.0.clear();
}

/// `OnEnter(Playing)`: убирает прошлую партию и ставит змейку в начальное положение.
/// Еду кладёт [`spawn_food`] в том же кадре.
pub(super) fn start_round(
    mut commands: Commands,
    cfg: Res<GameConfig>,
    entities: Query<Entity, Or<(With<SnakeSegment>, With<Food>)>>,
    mut body: ResMut<SnakeBody>,
    mut last_tail: ResMut<LastTail>,
    mut score: ResMut<Score>,
    mut timer: ResMut<MoveTimer>,
) {
    for e in &entities {
        commands.entity(e).despawn();
    }
    let cells = rules::initial_snake(&cfg.board, cfg.initial_len, cfg.start_direction);
    body.0 = cells
        .iter()
        .enumerate()
        .map(|(i, &cell)| {
            let mut segment = commands.spawn((SnakeSegment, GridPos(cell)));
            if i == 0 {
                segment.insert((SnakeHead, Heading::new(cfg.start_direction)));
            }
            segment.id()
        })
        .collect();
    last_tail.0 = None;
    score.0 = 0;
    *timer = MoveTimer::new(cfg.speed.interval(0));
}

pub(super) fn finish_round(
    mut round_over: MessageReader<RoundOver>,
    cfg: Res<GameConfig>,
    score: Res<Score>,
    mut best: ResMut<BestScore>,
    mut last_round: ResMut<LastRound>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Some(&RoundOver(end)) = round_over.read().last() else {
        return;
    };
    last_round.0 = Some(end);
    if score.0 > best.0 {
        best.0 = score.0;
        if let Some(path) = &cfg.best_score_file
            && let Err(err) = highscore::save(path, best.0)
        {
            warn!("не удалось сохранить рекорд в {}: {err}", path.display());
        }
    }
    next.set(GameState::GameOver);
}

// ---------- Ход ----------

pub(super) fn steer(keys: Res<ButtonInput<KeyCode>>, mut heads: Query<&mut Heading, With<SnakeHead>>) {
    let Ok(mut heading) = heads.single_mut() else {
        return;
    };
    for dir in keys.get_just_pressed().filter_map(|&k| key_direction(k)) {
        heading.request(dir);
    }
}

pub(super) fn advance_timer(time: Res<Time>, mut timer: ResMut<MoveTimer>) {
    timer.tick(time.delta());
}

pub(super) fn move_due(timer: Res<MoveTimer>) -> bool {
    timer.just_finished()
}

pub(super) fn move_snake(
    cfg: Res<GameConfig>,
    body: Res<SnakeBody>,
    mut last_tail: ResMut<LastTail>,
    mut heads: Query<&mut Heading, With<SnakeHead>>,
    mut positions: Query<&mut GridPos, With<SnakeSegment>>,
    mut round_over: MessageWriter<RoundOver>,
) {
    let Ok(mut heading) = heads.single_mut() else {
        return;
    };
    let Ok(cells) = body.0.iter().map(|&e| positions.get(e).map(|p| p.0)).collect::<Result<Vec<Cell>, _>>() else {
        return;
    };
    let Some(&head) = cells.first() else {
        return;
    };
    let next = head.step(heading.advance());
    if let Err(collision) = rules::check_move(&cfg.board, &cells, next) {
        round_over.write(RoundOver(match collision {
            Collision::Wall => RoundEnd::HitWall,
            Collision::Body => RoundEnd::HitSelf,
        }));
        return;
    }
    last_tail.0 = cells.last().copied();
    for (&e, cell) in body.0.iter().zip(iter::once(next).chain(cells)) {
        positions.get_mut(e).expect("сегмент проверен выше").0 = cell;
    }
}

pub(super) fn eat_food(
    mut commands: Commands,
    heads: Query<&GridPos, With<SnakeHead>>,
    food: Query<(Entity, &GridPos), With<Food>>,
    mut score: ResMut<Score>,
    mut eaten: MessageWriter<FoodEaten>,
) {
    let Ok(head) = heads.single() else {
        return;
    };
    for (e, pos) in &food {
        if pos == head {
            commands.entity(e).despawn();
            score.0 += 1;
            eaten.write(FoodEaten { at: pos.0 });
        }
    }
}

/// Новый сегмент встаёт на клетку, которую хвост освободил на этом ходу.
pub(super) fn grow_snake(
    mut commands: Commands,
    mut eaten: MessageReader<FoodEaten>,
    last_tail: Res<LastTail>,
    mut body: ResMut<SnakeBody>,
) {
    for _ in eaten.read() {
        if let Some(cell) = last_tail.0 {
            body.0.push(commands.spawn((SnakeSegment, GridPos(cell))).id());
        }
    }
}

pub(super) fn update_speed(cfg: Res<GameConfig>, score: Res<Score>, mut timer: ResMut<MoveTimer>) {
    timer.set_duration(cfg.speed.interval(score.0));
}

/// Кладёт еду, если её нет. Свободных клеток не осталось - партия выиграна.
pub(super) fn spawn_food(
    mut commands: Commands,
    cfg: Res<GameConfig>,
    body: Res<SnakeBody>,
    food: Query<(), With<Food>>,
    segments: Query<&GridPos, With<SnakeSegment>>,
    mut source: ResMut<FoodSource>,
    mut round_over: MessageWriter<RoundOver>,
) {
    if !food.is_empty() || body.0.is_empty() {
        return;
    }
    let occupied: Vec<Cell> = segments.iter().map(|p| p.0).collect();
    let free = rules::free_cells(&cfg.board, &occupied);
    match source.0.pick(&free) {
        Some(cell) => {
            commands.spawn((Food, GridPos(cell)));
        }
        None => {
            round_over.write(RoundOver(RoundEnd::BoardFilled));
        }
    }
}

// ---------- Переходы между экранами с клавиатуры ----------

pub(super) fn menu_keys(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        next.set(GameState::Playing);
    }
}

pub(super) fn pause_keys(
    keys: Res<ButtonInput<KeyCode>>,
    pause: Res<State<Pause>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::KeyP]) {
        next_pause.set(match pause.get() {
            Pause::Running => Pause::Paused,
            Pause::Paused => Pause::Running,
        });
    } else if *pause.get() == Pause::Paused && keys.just_pressed(KeyCode::KeyM) {
        next_game.set(GameState::Menu);
    }
}

pub(super) fn game_over_keys(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::KeyR]) {
        next.set(GameState::Playing);
    } else if keys.any_just_pressed([KeyCode::Escape, KeyCode::KeyM]) {
        next.set(GameState::Menu);
    }
}
