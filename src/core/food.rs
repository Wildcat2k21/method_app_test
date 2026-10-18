//! Выбор клетки для еды.

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::grid::Cell;

/// Источник позиций еды. Трейт позволяет подменить случайный выбор
/// детерминированным, например в тестах.
pub trait FoodSpawner: Send + Sync {
    /// Клетка из `free` для новой еды; `None`, если `free` пуст.
    fn pick(&mut self, free: &[Cell]) -> Option<Cell>;
}

/// Равновероятный выбор свободной клетки.
pub struct RandomSpawner {
    rng: StdRng,
}

impl RandomSpawner {
    /// Одинаковый `seed` даёт одинаковую последовательность выборов.
    pub fn seeded(seed: u64) -> Self {
        Self { rng: StdRng::seed_from_u64(seed) }
    }

    pub fn from_entropy() -> Self {
        Self::seeded(rand::random())
    }
}

impl FoodSpawner for RandomSpawner {
    fn pick(&mut self, free: &[Cell]) -> Option<Cell> {
        (!free.is_empty()).then(|| free[self.rng.random_range(0..free.len())])
    }
}
