use std::array;

use rand::{RngExt, rngs::ThreadRng, seq::IteratorRandom};

use crate::{
    models::{
        card::Card, direction::Direction, position::Position, tile::Tile, tile_card::TileCard,
    },
    utils::{
        constants::{BOARD_SIZE, TILE_TOTAL},
        helpers::{idx2pos, pos2idx},
    },
};

pub struct Board {
    tiles: [Tile; TILE_TOTAL],
}

impl Board {
    pub fn build(density: f64, rng: &mut ThreadRng) -> Self {
        Self {
            tiles: array::from_fn(|_| match rng.random_bool(density) {
                true => Tile::Block,
                false => Tile::Empty,
            }),
        }
    }

    pub fn get(&self, pos: Position) -> Option<&Tile> {
        let idx = pos2idx(pos);
        idx.and_then(|i| self.tiles.get(i))
    }

    pub fn get_relative(&self, pos: Position, dir: Direction) -> Option<&TileCard> {
        let next = pos.relative(
            dir,
            Position {
                row: BOARD_SIZE,
                col: BOARD_SIZE,
            },
        );

        next.and_then(|p| self.get(p)).and_then(|t| match t {
            Tile::Card(tc) => Some(tc),
            _ => None,
        })
    }

    pub fn row(&self, row: usize) -> &[Tile] {
        let lower = BOARD_SIZE * row;
        let upper = lower + BOARD_SIZE;
        self.tiles.get(lower..upper).unwrap_or_default()
    }

    pub fn place_card(
        &mut self,
        card: Card,
        target: Position,
        owner_id: u64,
    ) -> Result<(), String> {
        let Some(idx) = pos2idx(target) else {
            return Err(format!("invalid position {:?}", target));
        };

        self.tiles[idx] = Tile::Card(TileCard { owner_id, card });

        Ok(())
    }

    pub fn is_available(&self, pos: Position) -> bool {
        self.get(pos) == Some(&Tile::Empty)
    }

    pub fn find_available(&self, rng: &mut ThreadRng) -> Option<Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| match tile {
                Tile::Empty => idx2pos(idx),
                _ => None,
            })
            .choose(rng)
    }

    pub fn neighbors(&self, pos: Position) -> Vec<(&TileCard, Direction)> {
        (0..8u8)
            .filter_map(|k| Direction::try_from(k).ok())
            .filter_map(|dir| self.get_relative(pos, dir).map(|tc| (tc, dir)))
            .collect()
    }

    pub fn set_owner(&mut self, pos: Position, owner_id: u64) -> Result<(), String> {
        let Some(idx) = pos2idx(pos) else {
            return Err(format!("invalid pos {:?}", pos));
        };

        let tc = self
            .tiles
            .get_mut(idx)
            .and_then(|t| match t {
                Tile::Card(tc) => Some(tc),
                _ => None,
            })
            .ok_or("Tile is not a card".to_string())?;

        tc.owner_id = owner_id;
        Ok(())
    }

    pub fn find_placed_by_id(&self, card_id: u64) -> Option<(&TileCard, Position)> {
        self.tiles
            .iter()
            .enumerate()
            .find_map(|(idx, tile)| match tile {
                Tile::Card(tc) => {
                    if tc.card.id != card_id {
                        return None;
                    }

                    idx2pos(idx).map(|pos| (tc, pos))
                }
                _ => None,
            })
    }
}
