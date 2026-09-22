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

    pub fn get_card(&self, pos: Position) -> Option<&TileCard> {
        pos2idx(pos)
            .and_then(|i| self.tiles.get(i))
            .and_then(|t| match t {
                Tile::Card(tc) => Some(tc),
                _ => None,
            })
    }

    pub fn get_relative(&self, pos: Position, dir: &Direction) -> Option<&TileCard> {
        pos.relative(
            dir,
            Position {
                row: BOARD_SIZE,
                col: BOARD_SIZE,
            },
        )
        .and_then(|next| self.get_card(next))
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

    pub fn score(&self, owner_id: u64) -> usize {
        self.tiles
            .iter()
            .filter_map(|t| match t {
                Tile::Card(tile_card) => Some(tile_card),
                _ => None,
            })
            .filter(|tc| tc.owner_id == owner_id)
            .count()
    }

    pub fn count_empty(&self) -> usize {
        self.tiles
            .iter()
            .filter(|t| matches!(t, Tile::Empty))
            .count()
    }
}

#[cfg(test)]
impl Board {
    pub(crate) fn from_tiles(tiles: [Tile; TILE_TOTAL]) -> Self {
        Self { tiles }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{battle_class::BattleClass, tile::Tile},
        test_support::{card, empty_tiles},
    };

    #[test]
    fn construction_lookup_rows_and_counts_distinguish_tile_states() {
        let mut rng = rand::rng();
        let empty = Board::build(0.0, &mut rng);
        assert_eq!(empty.count_empty(), TILE_TOTAL);
        assert_eq!(empty.row(0).len(), BOARD_SIZE);
        assert!(empty.row(BOARD_SIZE).is_empty());

        let blocked = Board::build(1.0, &mut rng);
        assert_eq!(blocked.count_empty(), 0);
        assert!(matches!(
            blocked.get(Position::new(0, 0)),
            Some(Tile::Block)
        ));
        assert!(blocked.get(Position::new(BOARD_SIZE, 0)).is_none());
    }

    #[test]
    fn placement_lookup_ownership_scores_and_identity_are_observable() {
        let mut board = Board::from_tiles(empty_tiles());
        board
            .place_card(
                card(7, 0, BattleClass::Physical, 1, 2, 3),
                Position::new(1, 2),
                10,
            )
            .unwrap();

        assert!(!board.is_available(Position::new(1, 2)));
        assert_eq!(board.count_empty(), TILE_TOTAL - 1);
        assert_eq!(board.score(10), 1);
        let (placed, pos) = board.find_placed_by_id(7).unwrap();
        assert_eq!((pos.row, pos.col, placed.owner_id), (1, 2, 10));

        board.set_owner(Position::new(1, 2), 20).unwrap();
        assert_eq!(board.score(10), 0);
        assert_eq!(board.score(20), 1);
        assert!(board.set_owner(Position::new(0, 0), 20).is_err());
        assert!(
            board
                .place_card(
                    card(8, 0, BattleClass::Physical, 0, 0, 0),
                    Position::new(4, 0),
                    1
                )
                .is_err()
        );
    }

    #[test]
    fn current_low_level_placement_overwrites_non_empty_tiles() {
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Block;
        let mut board = Board::from_tiles(tiles);
        board
            .place_card(
                card(1, 0, BattleClass::Physical, 0, 0, 0),
                Position::new(0, 0),
                9,
            )
            .unwrap();
        assert!(board.get_card(Position::new(0, 0)).is_some());
    }
}
