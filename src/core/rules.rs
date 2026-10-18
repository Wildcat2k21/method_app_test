//! Правила хода: повороты, столкновения, свободные клетки, скорость.

use std::time::Duration;

use super::grid::{Board, Cell, Direction};

/// Во что врезалась голова.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collision {
    Wall,
    Body,
}

/// Новое направление, если поворот разрешён.
///
/// Запрещены разворот на 180° (голова въехала бы в шею) и повтор текущего
/// направления (не меняет ничего, но занял бы слот очереди ввода).
pub fn turn(current: Direction, requested: Direction) -> Option<Direction> {
    (requested != current && requested != current.opposite()).then_some(requested)
}

/// Проверяет ход головы в клетку `next`. `body[0]` - голова, последний элемент - хвост.
///
/// Хвост в этот же тик уходит со своей клетки, поэтому въезд в неё разрешён.
/// Рост на этом не сказывается: змейка растёт, только когда съела еду, а еда
/// никогда не лежит на клетке змейки.
pub fn check_move(board: &Board, body: &[Cell], next: Cell) -> Result<(), Collision> {
    if !board.contains(next) {
        return Err(Collision::Wall);
    }
    let blocking = &body[..body.len().saturating_sub(1)];
    if blocking.contains(&next) {
        return Err(Collision::Body);
    }
    Ok(())
}

/// Свободные клетки поля в построчном порядке. Клетки `occupied` вне поля игнорируются.
pub fn free_cells(board: &Board, occupied: &[Cell]) -> Vec<Cell> {
    let mut taken = vec![false; board.area()];
    for &cell in occupied.iter().filter(|&&c| board.contains(c)) {
        taken[board.index(cell)] = true;
    }
    board.cells().filter(|&c| !taken[board.index(c)]).collect()
}

/// Начальная змейка: голова в центре поля, тело тянется против `dir`.
/// Сегменты, не поместившиеся на поле, отбрасываются.
pub fn initial_snake(board: &Board, len: usize, dir: Direction) -> Vec<Cell> {
    let back = dir.opposite();
    std::iter::successors(Some(board.center()), |c| Some(c.step(back)))
        .take(len)
        .take_while(|&c| board.contains(c))
        .collect()
}

/// Скорость растёт со счётом: каждые `every` очков интервал хода уменьшается на `step`,
/// но не опускается ниже `min`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeedRules {
    pub base: Duration,
    pub min: Duration,
    pub step: Duration,
    pub every: u32,
}

impl SpeedRules {
    pub fn interval(&self, score: u32) -> Duration {
        let level = score.checked_div(self.every).unwrap_or(0);
        self.base.saturating_sub(self.step.saturating_mul(level)).max(self.min)
    }

    /// Уровень скорости для HUD: 1 на старте.
    pub fn level(&self, score: u32) -> u32 {
        score.checked_div(self.every).unwrap_or(0) + 1
    }
}

impl Default for SpeedRules {
    fn default() -> Self {
        Self {
            base: Duration::from_millis(150),
            min: Duration::from_millis(60),
            step: Duration::from_millis(10),
            every: 5,
        }
    }
}
