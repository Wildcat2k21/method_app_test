//! `core::rules::check_move` - покрытие ветвей и условий.
//!
//! ```text
//! B1  if !board.contains(next)          → Err(Wall)
//!       C1 x ≥ 0   C2 x < width   C3 y ≥ 0   C4 y < height   (тест b1_cN - условие CN ложно)
//! B2  blocking = body[..len-1]          (C5: len == 0 → saturating_sub)
//! B3  if blocking.contains(&next)       → Err(Body)
//! B4  иначе                             → Ok(())
//! ```

use snake_game::core::rules::check_move;
use snake_game::core::{Board, Cell, Collision};

const BOARD: Board = Board::new(5, 4);

fn cells(list: &[(i32, i32)]) -> Vec<Cell> {
    list.iter().map(|&(x, y)| Cell::new(x, y)).collect()
}

// ---------- B1: стена, по одному тесту на каждое условие ----------

#[test]
fn b1_c1_left_wall() {
    let body = cells(&[(0, 1), (1, 1)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(-1, 1)), Err(Collision::Wall));
}

#[test]
fn b1_c2_right_wall() {
    let body = cells(&[(4, 1), (3, 1)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(5, 1)), Err(Collision::Wall));
}

#[test]
fn b1_c3_bottom_wall() {
    let body = cells(&[(2, 0), (2, 1)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(2, -1)), Err(Collision::Wall));
}

#[test]
fn b1_c4_top_wall() {
    let body = cells(&[(2, 3), (2, 2)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(2, 4)), Err(Collision::Wall));
}

/// Граница условий C1–C4 с другой стороны: крайние клетки поля - не стена.
#[test]
fn b1_false_on_every_corner() {
    for corner in cells(&[(0, 0), (4, 0), (0, 3), (4, 3)]) {
        assert_eq!(check_move(&BOARD, &[], corner), Ok(()), "{corner:?}");
    }
}

// ---------- B3: тело ----------

/// Голова поворачивает в сегмент посередине тела.
#[test]
fn b3_true_hits_body() {
    // Кольцо: голова (1,1) → вниз в (1,0), который занят сегментом.
    let body = cells(&[(1, 1), (2, 1), (2, 0), (1, 0), (0, 0)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(1, 0)), Err(Collision::Body));
}

/// Хвост в этот ход уходит: въезд в его клетку - не столкновение (B3 ложно).
#[test]
fn b3_false_tail_cell_is_free() {
    let body = cells(&[(1, 1), (2, 1), (2, 0), (1, 0)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(1, 0)), Ok(()));
}

// ---------- B2/C5: длина тела ----------

#[test]
fn c5_empty_body_does_not_panic() {
    assert_eq!(check_move(&BOARD, &[], Cell::new(2, 2)), Ok(()));
}

/// Змейка из одной клетки: blocking пуст, любая клетка внутри поля свободна.
#[test]
fn c5_single_segment() {
    let body = cells(&[(2, 2)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(2, 3)), Ok(()));
}

// ---------- B4 ----------

#[test]
fn b4_free_cell() {
    let body = cells(&[(2, 2), (1, 2), (0, 2)]);
    assert_eq!(check_move(&BOARD, &body, Cell::new(3, 2)), Ok(()));
}
