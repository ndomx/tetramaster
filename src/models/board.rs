use std::array;

use rand::{RngExt, rngs::ThreadRng};

use crate::{constants::BOARD_SIZE, models::{direction::Direction, position::Position, tile::Tile}};

pub struct Board {
    tiles: [[Tile; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn build(density: f64, rng: &mut ThreadRng) -> Self {
        Self {
            tiles: array::from_fn(|_| {
                array::from_fn(|_| match rng.random_bool(density) {
                    true => Tile::Block,
                    false => Tile::Empty,
                })
            }),
        }
    }

    pub fn get(&self, pos: Position) -> Option<&Tile> {
        let row_opt = self.tiles.get(pos.row);
        if let Some(row) = row_opt {
            return row.get(pos.col);
        }

        None
    }

    pub fn get_relative(&self, pos: Position, dir: &Direction) -> Option<&Tile> {
        let next = pos.relative(
            dir,
            Position {
                row: BOARD_SIZE,
                col: BOARD_SIZE,
            },
        );

        next.and_then(|p| self.get(p))
    }

    fn is_available(&self, pos: Position) -> bool {
        self.get(pos) == Some(&Tile::Empty)
    }
}
