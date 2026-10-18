//! Клетки, поле и направления.

/// Клетка поля. `(0, 0)` - левый нижний угол, `y` растёт вверх.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Соседняя клетка в направлении `dir`. Может оказаться за пределами поля.
    pub fn step(self, dir: Direction) -> Self {
        let (dx, dy) = dir.delta();
        Self::new(self.x + dx, self.y + dy)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const ALL: [Direction; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, 1),
            Self::Down => (0, -1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Прямоугольное поле `width × height` клеток.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub width: i32,
    pub height: i32,
}

impl Board {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn contains(&self, cell: Cell) -> bool {
        (0..self.width).contains(&cell.x) && (0..self.height).contains(&cell.y)
    }

    /// Число клеток поля; для поля с неположительной стороной - 0.
    pub fn area(&self) -> usize {
        self.width.max(0) as usize * self.height.max(0) as usize
    }

    /// Центр поля; для чётной стороны - клетка левее/ниже середины.
    pub fn center(&self) -> Cell {
        Cell::new((self.width - 1) / 2, (self.height - 1) / 2)
    }

    /// Индекс клетки в построчном порядке. Вызывать только для клеток внутри поля.
    pub fn index(&self, cell: Cell) -> usize {
        (cell.y * self.width + cell.x) as usize
    }

    /// Все клетки построчно, снизу вверх.
    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.height).flat_map(move |y| (0..self.width).map(move |x| Cell::new(x, y)))
    }
}
