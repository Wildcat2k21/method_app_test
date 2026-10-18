use bevy::prelude::*;

use crate::core::{Cell, Direction, rules};

/// Позиция на поле в клетках. Пиксельный `Transform` из неё выводит [`super::view`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct GridPos(pub Cell);

/// Любой сегмент змейки, включая голову. Порядок сегментов - в [`super::SnakeBody`].
#[derive(Component, Debug)]
pub struct SnakeSegment;

#[derive(Component, Debug)]
pub struct SnakeHead;

#[derive(Component, Debug)]
pub struct Food;

/// Направление головы. `current` - куда змейка шла на прошлом ходу,
/// `queued` - поворот, который применится на следующем.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heading {
    pub current: Direction,
    pub queued: Option<Direction>,
}

impl Heading {
    pub fn new(dir: Direction) -> Self {
        Self { current: dir, queued: None }
    }

    /// Ставит поворот в очередь, если он разрешён относительно `current`.
    /// Сверка с `current`, а не с `queued`, не даёт развернуться двумя быстрыми
    /// нажатиями за один ход (вправо → вверх → влево).
    pub fn request(&mut self, dir: Direction) -> bool {
        let accepted = rules::turn(self.current, dir);
        if accepted.is_some() {
            self.queued = accepted;
        }
        accepted.is_some()
    }

    /// Применяет поворот из очереди; возвращает направление этого хода.
    pub fn advance(&mut self) -> Direction {
        if let Some(dir) = self.queued.take() {
            self.current = dir;
        }
        self.current
    }
}
