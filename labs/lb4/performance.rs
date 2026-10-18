//! Бюджет: один кадр при 60 FPS - 16.7 мс. Ход змейки (тик) должен укладываться
//! в него с большим запасом, иначе игра начнёт пропускать кадры.

use std::hint::black_box;
use std::time::{Duration, Instant};

use snake_game::core::rules::free_cells;
use snake_game::core::{Board, Cell};

use crate::common::{Harness, config};

const FRAME: Duration = Duration::from_micros(16_667);

/// Среднее время `f` за `runs` запусков.
fn measure(runs: u32, mut f: impl FnMut()) -> Duration {
    let start = Instant::now();
    for _ in 0..runs {
        f();
    }
    start.elapsed() / runs
}

/// Smoke-проверка: на стандартном поле средний ход (с едой и ростом) короче кадра
/// даже в отладочной сборке.
#[test]
fn tick_fits_frame_budget_on_default_board() {
    let food = (10..19).map(|x| Cell::new(x, 7));
    let mut h = Harness::playing(config(20, 15), food);
    let per_tick = measure(1, || {
        for _ in 0..9 {
            h.tick();
        }
    }) / 9;
    assert!(per_tick < FRAME, "ход занял {per_tick:?}");
}

/// Нагрузка: время `free_cells` от размера поля при змейке на половине клеток.
/// Ожидаемая сложность O(площадь) - время должно расти пропорционально числу клеток.
#[test]
#[ignore = "замер: запускать в --release с --ignored --nocapture"]
fn free_cells_scaling() {
    println!("{:>11} {:>10} {:>12} {:>12}", "поле", "клеток", "free_cells", "нс/клетку");
    for side in [20, 50, 100, 200, 500, 1000] {
        let board = Board::new(side, side);
        let snake: Vec<Cell> = board.cells().step_by(2).collect();
        let t = measure(20, || {
            black_box(free_cells(black_box(&board), black_box(&snake)));
        });
        let per_cell = t.as_nanos() as f64 / board.area() as f64;
        println!("{:>11} {:>10} {:>12.3?} {:>12.2}", format!("{side}×{side}"), board.area(), t, per_cell);
    }
}

/// Стресс: на каком размере поля один `free_cells` (вызывается при каждом съедании)
/// перестаёт укладываться в кадр.
#[test]
#[ignore = "замер: запускать в --release с --ignored --nocapture"]
fn free_cells_frame_limit() {
    let mut side = 100;
    loop {
        let board = Board::new(side, side);
        let snake: Vec<Cell> = board.cells().step_by(2).collect();
        let t = measure(5, || {
            black_box(free_cells(&board, &snake));
        });
        println!("{side}×{side}: {t:?}");
        if t > FRAME || side >= 8000 {
            println!("предел: поле {side}×{side} ({} клеток)", board.area());
            break;
        }
        side *= 2;
    }
}
