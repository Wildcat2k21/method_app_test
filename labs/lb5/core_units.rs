//! ЛР5. Модульные тесты ядра правил (`crate::core`).
//!
//! Файл лежит в labs/lb5, но компилируется как дочерний модуль `crate::core`
//! (подключён через `#[path]` в src/core/mod.rs). Поэтому тестам доступно всё,
//! что видно внутри модуля, включая непубличное, - это и отличает модульные тесты
//! Rust от интеграционных (labs/lbN/main.rs видят только публичный API).
//!
//! Каждый тест - по схеме Arrange / Act / Assert.

use super::*;
use super::rules::{free_cells, initial_snake, turn};

// ---------- grid ----------

#[test]
fn step_moves_by_one_cell() {
    let c = Cell::new(3, 3);
    assert_eq!(c.step(Direction::Up), Cell::new(3, 4));
    assert_eq!(c.step(Direction::Down), Cell::new(3, 2));
    assert_eq!(c.step(Direction::Left), Cell::new(2, 3));
    assert_eq!(c.step(Direction::Right), Cell::new(4, 3));
}

#[test]
fn opposite_is_involution_and_steps_back() {
    for d in Direction::ALL {
        assert_eq!(d.opposite().opposite(), d);
        assert_eq!(Cell::new(0, 0).step(d).step(d.opposite()), Cell::new(0, 0));
    }
}

#[test]
fn board_cells_cover_area_in_index_order() {
    let board = Board::new(4, 3);
    let cells: Vec<Cell> = board.cells().collect();
    assert_eq!(cells.len(), board.area());
    for (i, &c) in cells.iter().enumerate() {
        assert_eq!(board.index(c), i);
    }
}

#[test]
fn degenerate_board_has_no_cells() {
    assert_eq!(Board::new(0, 5).area(), 0);
    assert_eq!(Board::new(-3, 5).area(), 0);
    assert_eq!(Board::new(0, 5).cells().count(), 0);
}

// ---------- rules ----------

#[test]
fn turn_allows_only_perpendicular() {
    assert_eq!(turn(Direction::Right, Direction::Up), Some(Direction::Up));
    assert_eq!(turn(Direction::Right, Direction::Down), Some(Direction::Down));
    assert_eq!(turn(Direction::Right, Direction::Right), None);
    assert_eq!(turn(Direction::Right, Direction::Left), None);
}

#[test]
fn initial_snake_trails_behind_head() {
    let snake = initial_snake(&Board::new(20, 15), 3, Direction::Right);
    assert_eq!(snake, vec![Cell::new(9, 7), Cell::new(8, 7), Cell::new(7, 7)]);
}

#[test]
fn initial_snake_is_clipped_by_board() {
    let snake = initial_snake(&Board::new(4, 1), 10, Direction::Right);
    assert_eq!(snake, vec![Cell::new(1, 0), Cell::new(0, 0)]);
}

#[test]
fn free_cells_excludes_snake_and_ignores_outside() {
    let board = Board::new(2, 2);
    let free = free_cells(&board, &[Cell::new(0, 0), Cell::new(5, 5)]);
    assert_eq!(free, vec![Cell::new(1, 0), Cell::new(0, 1), Cell::new(1, 1)]);
}

// ---------- food ----------

#[test]
fn random_spawner_picks_only_from_free() {
    let free = [Cell::new(1, 1), Cell::new(2, 2)];
    let mut spawner = RandomSpawner::seeded(5);
    for _ in 0..100 {
        assert!(free.contains(&spawner.pick(&free).unwrap()));
    }
}

#[test]
fn random_spawner_none_when_board_full() {
    assert_eq!(RandomSpawner::seeded(5).pick(&[]), None);
}

#[test]
fn random_spawner_is_reproducible_by_seed() {
    let free: Vec<Cell> = Board::new(10, 10).cells().collect();
    let run = |seed| {
        let mut s = RandomSpawner::seeded(seed);
        (0..20).map(|_| s.pick(&free).unwrap()).collect::<Vec<_>>()
    };
    assert_eq!(run(1), run(1));
    assert_ne!(run(1), run(2));
}
