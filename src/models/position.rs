use crate::models::direction::Direction;

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn relative(&self, direction: &Direction, bounds: Self) -> Option<Self> {
        match direction {
            Direction::N => self.plus(-1, 0, bounds),
            Direction::NE => self.plus(-1, 1, bounds),
            Direction::E => self.plus(0, 1, bounds),
            Direction::SE => self.plus(1, 1, bounds),
            Direction::S => self.plus(1, 0, bounds),
            Direction::SW => self.plus(1, -1, bounds),
            Direction::W => self.plus(0, -1, bounds),
            Direction::NW => self.plus(-1, -1, bounds),
        }
    }

    fn plus(&self, x: isize, y: isize, bounds: Self) -> Option<Self> {
        let row = x + (self.row as isize);
        let col = y + (self.col as isize);

        if row < 0 || row >= (bounds.row as isize) {
            return None;
        }

        if col < 0 || col >= (bounds.col as isize) {
            return None;
        }

        Some(Self {
            row: row as usize,
            col: col as usize,
        })
    }
}
