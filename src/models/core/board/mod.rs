use std::array;
use std::fmt::{Display, Formatter};

use rand::{RngExt, rngs::ThreadRng};

mod board_card;
mod tile;

pub use board_card::BoardCard;
pub use tile::Tile;

use crate::{
    models::core::{
        card::Card,
        geometry::{Direction, Position},
    },
    utils::{
        constants::{BOARD_SIZE, TILE_TOTAL},
        helpers::{idx2pos, pos2idx},
    },
};

pub struct Board {
    tiles: [Tile; TILE_TOTAL],
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlacementError {
    OutOfBounds(Position),
    Blocked(Position),
    Occupied(Position),
}

impl Display for PlacementError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(position) => {
                write!(formatter, "position {position:?} is out of bounds")
            }
            Self::Blocked(position) => write!(formatter, "position {position:?} is blocked"),
            Self::Occupied(position) => write!(formatter, "position {position:?} is occupied"),
        }
    }
}

impl std::error::Error for PlacementError {}

impl Board {
    pub fn build(density: f64, rng: &mut ThreadRng) -> Self {
        Self {
            tiles: array::from_fn(|_| match rng.random_bool(density) {
                true => Tile::Blocked,
                false => Tile::Empty,
            }),
        }
    }

    pub fn get(&self, pos: Position) -> Option<&Tile> {
        let idx = pos2idx(pos);
        idx.and_then(|i| self.tiles.get(i))
    }

    pub fn get_card(&self, pos: Position) -> Option<&BoardCard> {
        pos2idx(pos)
            .and_then(|i| self.tiles.get(i))
            .and_then(|t| match t {
                Tile::Occupied(tc) => Some(tc),
                _ => None,
            })
    }

    pub fn get_relative(&self, pos: Position, dir: &Direction) -> Option<&BoardCard> {
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
    ) -> Result<(), PlacementError> {
        let idx = pos2idx(target).ok_or(PlacementError::OutOfBounds(target))?;

        match &self.tiles[idx] {
            Tile::Empty => {}
            Tile::Blocked => return Err(PlacementError::Blocked(target)),
            Tile::Occupied(_) => return Err(PlacementError::Occupied(target)),
        }

        self.tiles[idx] = Tile::Occupied(BoardCard { owner_id, card });

        Ok(())
    }

    pub fn is_available(&self, pos: Position) -> bool {
        self.get(pos) == Some(&Tile::Empty)
    }

    pub fn set_owner(&mut self, pos: Position, owner_id: u64) -> Result<(), String> {
        let Some(idx) = pos2idx(pos) else {
            return Err(format!("invalid pos {:?}", pos));
        };

        let tc = self
            .tiles
            .get_mut(idx)
            .and_then(|t| match t {
                Tile::Occupied(tc) => Some(tc),
                _ => None,
            })
            .ok_or("Tile is not a card".to_string())?;

        tc.owner_id = owner_id;
        Ok(())
    }

    pub fn position_of_card(&self, card_id: u64) -> Option<(&BoardCard, Position)> {
        self.tiles
            .iter()
            .enumerate()
            .find_map(|(idx, tile)| match tile {
                Tile::Occupied(tc) => {
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
                Tile::Occupied(board_card) => Some(board_card),
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
        models::core::card::BattleClass,
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
            Some(Tile::Blocked)
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
        let (placed, pos) = board.position_of_card(7).unwrap();
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
    fn failed_placement_is_atomic_for_every_invalid_target() {
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Blocked;
        tiles[1] = Tile::Occupied(BoardCard {
            owner_id: 4,
            card: card(2, 0, BattleClass::Physical, 0, 0, 0),
        });
        let mut board = Board::from_tiles(tiles);

        assert_eq!(
            board.place_card(
                card(3, 0, BattleClass::Physical, 0, 0, 0),
                Position::new(BOARD_SIZE, 0),
                9,
            ),
            Err(PlacementError::OutOfBounds(Position::new(BOARD_SIZE, 0)))
        );
        assert_eq!(board.count_empty(), TILE_TOTAL - 2);

        assert_eq!(
            board.place_card(
                card(4, 0, BattleClass::Physical, 0, 0, 0),
                Position::new(0, 0),
                9,
            ),
            Err(PlacementError::Blocked(Position::new(0, 0)))
        );
        assert!(matches!(
            board.get(Position::new(0, 0)),
            Some(Tile::Blocked)
        ));

        assert_eq!(
            board.place_card(
                card(5, 0, BattleClass::Physical, 0, 0, 0),
                Position::new(0, 1),
                9,
            ),
            Err(PlacementError::Occupied(Position::new(0, 1)))
        );
        let occupied = board.get_card(Position::new(0, 1)).unwrap();
        assert_eq!((occupied.owner_id, occupied.card.id), (4, 2));
    }
}
