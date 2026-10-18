//! Сценарии ручного чек-листа, переведённые в автотесты. Номер в имени - номер
//! строки чек-листа (MT-1…) в README.md.

use bevy::prelude::KeyCode;

use snake_game::core::{Cell, Direction};
use snake_game::game::{GameState, Pause, RoundEnd};

use crate::common::{Harness, config};

/// Еда в углу - вдали от маршрута змейки, чтобы не мешала сценарию.
const FAR: Cell = Cell::new(0, 0);

#[test]
fn mt1_enter_starts_round() {
    let mut h = Harness::new(config(20, 15), [FAR]);
    assert_eq!(h.state(), GameState::Menu);

    h.start();

    assert_eq!(h.state(), GameState::Playing);
    assert_eq!(h.snake(), vec![Cell::new(9, 7), Cell::new(8, 7), Cell::new(7, 7)]);
    assert_eq!(h.food(), Some(FAR));
    assert_eq!(h.score(), 0);
}

#[test]
fn mt2_snake_moves_one_cell_per_tick() {
    let mut h = Harness::playing(config(20, 15), [FAR]);

    h.advance(h.interval() / 2);
    assert_eq!(h.head(), Cell::new(9, 7), "полхода - змейка стоит");

    h.tick();
    assert_eq!(h.head(), Cell::new(10, 7));
    h.tick();
    assert_eq!(h.head(), Cell::new(11, 7));
}

#[test]
fn mt3_arrow_turns_snake() {
    let mut h = Harness::playing(config(20, 15), [FAR]);

    h.turn(Direction::Up);
    h.tick();

    assert_eq!(h.head(), Cell::new(9, 8));
}

#[test]
fn mt4_reverse_is_ignored() {
    let mut h = Harness::playing(config(20, 15), [FAR]);

    h.turn(Direction::Left);
    h.tick();

    assert_eq!(h.head(), Cell::new(10, 7), "разворот на 180° игнорируется");
}

#[test]
fn mt5_eating_grows_and_scores() {
    let mut h = Harness::playing(config(20, 15), [Cell::new(11, 7)]);

    h.ticks(2);

    assert_eq!(h.score(), 1);
    assert_eq!(h.snake().len(), 4);
    assert_ne!(h.food(), Some(Cell::new(11, 7)), "еда появилась заново в другом месте");
}

#[test]
fn mt6_pause_freezes_and_resumes() {
    let mut h = Harness::playing(config(20, 15), [FAR]);

    h.press(KeyCode::KeyP);
    h.frame();
    assert_eq!(h.pause(), Some(Pause::Paused));
    for _ in 0..10 {
        h.advance(h.interval());
    }
    assert_eq!(h.head(), Cell::new(9, 7), "на паузе змейка стоит");

    h.press(KeyCode::Escape);
    h.frame();
    assert_eq!(h.pause(), Some(Pause::Running));
    h.tick();
    assert_eq!(h.head(), Cell::new(10, 7));
}

#[test]
fn mt7_wall_ends_round() {
    let mut h = Harness::playing(config(20, 15), [FAR]);

    h.ticks(10); // голова x: 9 → 19, последняя клетка ряда
    assert_eq!(h.state(), GameState::Playing);
    h.tick();
    h.frame(); // переход в GameOver применяется в следующем кадре

    assert_eq!(h.state(), GameState::GameOver);
    assert_eq!(h.last_round(), Some(RoundEnd::HitWall));
}

#[test]
fn mt8_enter_restarts_after_game_over() {
    let mut h = Harness::playing(config(20, 15), [Cell::new(10, 7)]);
    h.ticks(11);
    h.frame();
    assert_eq!(h.state(), GameState::GameOver);

    h.start();

    assert_eq!(h.state(), GameState::Playing);
    assert_eq!(h.score(), 0);
    assert_eq!(h.snake().len(), 3);
    assert_eq!(h.head(), Cell::new(9, 7));
}

#[test]
fn mt9_menu_from_pause() {
    let mut h = Harness::playing(config(20, 15), [FAR]);
    h.press(KeyCode::KeyP);
    h.frame();

    h.press(KeyCode::KeyM);
    h.frame();

    assert_eq!(h.state(), GameState::Menu);
    assert!(h.snake().is_empty());
    assert_eq!(h.food(), None);
}
