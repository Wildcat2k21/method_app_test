//! Цепочки систем: move_snake → eat_food → grow_snake → update_speed → spawn_food → finish_round.

use std::time::Duration;

use snake_game::core::{Board, Cell, Direction};
use snake_game::game::{GameConfig, GameState, RoundEnd};

use crate::common::{Harness, config};

/// FR-4: move_snake запоминает освободившуюся клетку хвоста (LastTail),
/// grow_snake ставит туда новый сегмент после FoodEaten от eat_food.
#[test]
fn fr4_new_segment_appears_where_tail_was() {
    let mut h = Harness::playing(config(20, 15), [Cell::new(10, 7)]);
    let tail_before = *h.snake().last().unwrap();

    h.tick();

    let snake = h.snake();
    assert_eq!(snake.len(), 4);
    assert_eq!(snake[0], Cell::new(10, 7), "голова на месте еды");
    assert_eq!(*snake.last().unwrap(), tail_before, "новый сегмент - на клетке старого хвоста");
}

/// FR-5: eat_food меняет Score, update_speed перестраивает MoveTimer.
#[test]
fn fr5_speed_increases_every_five_points() {
    let food = (10..=14).map(|x| Cell::new(x, 7));
    let mut h = Harness::playing(config(20, 15), food);
    assert_eq!(h.interval(), Duration::from_millis(150));

    h.ticks(4);
    assert_eq!(h.score(), 4);
    assert_eq!(h.interval(), Duration::from_millis(150));

    h.tick();
    assert_eq!(h.score(), 5);
    assert_eq!(h.interval(), Duration::from_millis(140));
}

/// FR-6: move_snake пишет RoundOver(HitSelf), finish_round переводит игру в GameOver.
#[test]
fn fr6_biting_self_ends_round() {
    let cfg = GameConfig { initial_len: 5, ..config(20, 15) };
    let mut h = Harness::playing(cfg, [Cell::new(0, 0)]);

    for dir in [Direction::Up, Direction::Left, Direction::Down] {
        h.turn(dir);
        h.tick();
    }
    h.frame();

    assert_eq!(h.state(), GameState::GameOver);
    assert_eq!(h.last_round(), Some(RoundEnd::HitSelf));
}

/// FR-7: spawn_food не находит свободной клетки → RoundOver(BoardFilled) → победа.
#[test]
fn fr7_filling_board_wins() {
    let cfg = GameConfig { board: Board::new(3, 1), initial_len: 2, ..config(3, 1) };
    let mut h = Harness::playing(cfg, []);
    assert_eq!(h.food(), Some(Cell::new(2, 0)), "единственная свободная клетка");

    h.tick();
    h.frame();

    assert_eq!(h.snake().len(), 3);
    assert_eq!(h.state(), GameState::GameOver);
    assert_eq!(h.last_round(), Some(RoundEnd::BoardFilled));
}

/// FR-8: finish_round переносит счёт в рекорд, start_round обнуляет только счёт.
#[test]
fn fr8_best_score_survives_restart() {
    let mut h = Harness::playing(config(20, 15), [Cell::new(10, 7), Cell::new(11, 7)]);
    h.ticks(11);
    h.frame();
    assert_eq!((h.score(), h.best()), (2, 2));

    h.start();

    assert_eq!((h.score(), h.best()), (0, 2));
}

/// Еда никогда не появляется на змейке: spawn_food видит сегмент, добавленный
/// grow_snake в этом же кадре.
#[test]
fn food_never_spawns_on_snake() {
    // Сценарий просит положить еду на клетку, которую занимает новый сегмент.
    let mut h = Harness::playing(config(20, 15), [Cell::new(10, 7), Cell::new(7, 7)]);

    h.tick();

    let food = h.food().expect("еда на поле");
    assert!(!h.snake().contains(&food), "еда {food:?} на змейке {:?}", h.snake());
}
