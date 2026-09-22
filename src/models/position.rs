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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_directions_move_from_an_interior_position() {
        let origin = Position::new(1, 1);
        let bounds = Position::new(3, 3);
        let cases = [
            (Direction::N, (0, 1)),
            (Direction::NE, (0, 2)),
            (Direction::E, (1, 2)),
            (Direction::SE, (2, 2)),
            (Direction::S, (2, 1)),
            (Direction::SW, (2, 0)),
            (Direction::W, (1, 0)),
            (Direction::NW, (0, 0)),
        ];
        for (direction, expected) in cases {
            let actual = origin.relative(&direction, bounds).unwrap();
            assert_eq!((actual.row, actual.col), expected);
        }
    }

    #[test]
    fn relative_movement_rejects_every_edge_overflow() {
        let bounds = Position::new(2, 2);
        assert!(
            Position::new(0, 0)
                .relative(&Direction::N, bounds)
                .is_none()
        );
        assert!(
            Position::new(0, 0)
                .relative(&Direction::W, bounds)
                .is_none()
        );
        assert!(
            Position::new(1, 1)
                .relative(&Direction::S, bounds)
                .is_none()
        );
        assert!(
            Position::new(1, 1)
                .relative(&Direction::E, bounds)
                .is_none()
        );
    }
}
