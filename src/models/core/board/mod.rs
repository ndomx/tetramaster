use std::array;
use std::fmt::{Display, Formatter};

mod board_card;
mod board_side;
mod tile;

pub use board_card::BoardCard;
pub use board_side::BoardSide;
pub use tile::Tile;

use crate::{
    models::core::{
        card::Card,
        geometry::{Direction, Position},
    },
    utils::{
        constants::{BOARD_SIZE, MAX_BLOCKS, TILE_TOTAL},
        helpers::{idx2pos, pos2idx},
        random::GameRng,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardControlError {
    OutOfBounds(Position),
    NotOccupied(Position),
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

impl Display for BoardControlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(position) => {
                write!(formatter, "position {position:?} is out of bounds")
            }
            Self::NotOccupied(position) => {
                write!(formatter, "position {position:?} is not occupied")
            }
        }
    }
}

impl std::error::Error for BoardControlError {}

impl Board {
    pub fn build(rng: &mut GameRng) -> Self {
        let block_count = (0..MAX_BLOCKS).filter(|_| rng.random_bool(0.5)).count();
        let positions: [usize; TILE_TOTAL] = array::from_fn(|index| index);
        let mut tiles = array::from_fn(|_| Tile::Empty);

        for &index in rng.choose_multiple(&positions, block_count) {
            tiles[index] = Tile::Blocked;
        }

        Self { tiles }
    }

    pub fn get(&self, position: Position) -> Option<&Tile> {
        let index = pos2idx(position);
        index.and_then(|index| self.tiles.get(index))
    }

    pub fn get_card(&self, position: Position) -> Option<&BoardCard> {
        pos2idx(position)
            .and_then(|index| self.tiles.get(index))
            .and_then(|tile| match tile {
                Tile::Occupied(board_card) => Some(board_card),
                _ => None,
            })
    }

    pub fn get_relative(&self, position: Position, direction: &Direction) -> Option<&BoardCard> {
        position
            .relative(
                direction,
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
        controller: BoardSide,
    ) -> Result<(), PlacementError> {
        let index = pos2idx(target).ok_or(PlacementError::OutOfBounds(target))?;

        match &self.tiles[index] {
            Tile::Empty => {}
            Tile::Blocked => return Err(PlacementError::Blocked(target)),
            Tile::Occupied(_) => return Err(PlacementError::Occupied(target)),
        }

        self.tiles[index] = Tile::Occupied(BoardCard { controller, card });

        Ok(())
    }

    pub fn is_available(&self, position: Position) -> bool {
        self.get(position) == Some(&Tile::Empty)
    }

    pub fn set_controller(
        &mut self,
        position: Position,
        controller: BoardSide,
    ) -> Result<(), BoardControlError> {
        let Some(index) = pos2idx(position) else {
            return Err(BoardControlError::OutOfBounds(position));
        };

        let board_card = self
            .tiles
            .get_mut(index)
            .and_then(|tile| match tile {
                Tile::Occupied(board_card) => Some(board_card),
                _ => None,
            })
            .ok_or(BoardControlError::NotOccupied(position))?;

        board_card.controller = controller;
        Ok(())
    }

    pub fn position_of_card(&self, card_id: u64) -> Option<(&BoardCard, Position)> {
        self.tiles
            .iter()
            .enumerate()
            .find_map(|(index, tile)| match tile {
                Tile::Occupied(board_card) => {
                    if board_card.card.id != card_id {
                        return None;
                    }

                    idx2pos(index).map(|position| (board_card, position))
                }
                _ => None,
            })
    }

    pub fn score(&self, controller: BoardSide) -> usize {
        self.tiles
            .iter()
            .filter_map(|tile| match tile {
                Tile::Occupied(board_card) => Some(board_card),
                _ => None,
            })
            .filter(|board_card| board_card.controller == controller)
            .count()
    }

    pub fn count_empty(&self) -> usize {
        self.tiles
            .iter()
            .filter(|tile| matches!(tile, Tile::Empty))
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
        for seed in 0..64 {
            let board = Board::build(&mut GameRng::from_seed(seed));
            let block_count = TILE_TOTAL - board.count_empty();

            assert!(block_count <= MAX_BLOCKS, "seed {seed}");
            assert_eq!(board.row(0).len(), BOARD_SIZE);
            assert!(board.row(BOARD_SIZE).is_empty());
            assert!(board.get(Position::new(BOARD_SIZE, 0)).is_none());
        }
    }

    #[test]
    fn placement_lookup_control_scores_and_identity_are_observable() {
        let mut board = Board::from_tiles(empty_tiles());
        board
            .place_card(
                card(7, 0, BattleClass::Physical, 1, 2, 3),
                Position::new(1, 2),
                BoardSide::Blue,
            )
            .unwrap();

        assert!(!board.is_available(Position::new(1, 2)));
        assert_eq!(board.count_empty(), TILE_TOTAL - 1);
        assert_eq!(board.score(BoardSide::Blue), 1);
        let (placed, position) = board.position_of_card(7).unwrap();
        assert_eq!(
            (position.row, position.col, placed.controller),
            (1, 2, BoardSide::Blue)
        );

        board
            .set_controller(Position::new(1, 2), BoardSide::Red)
            .unwrap();
        assert_eq!(board.score(BoardSide::Blue), 0);
        assert_eq!(board.score(BoardSide::Red), 1);
        assert_eq!(
            board.set_controller(Position::new(0, 0), BoardSide::Red),
            Err(BoardControlError::NotOccupied(Position::new(0, 0)))
        );
        assert_eq!(
            board.set_controller(Position::new(BOARD_SIZE, 0), BoardSide::Red),
            Err(BoardControlError::OutOfBounds(Position::new(BOARD_SIZE, 0)))
        );
        assert!(
            board
                .place_card(
                    card(8, 0, BattleClass::Physical, 0, 0, 0),
                    Position::new(4, 0),
                    BoardSide::Blue
                )
                .is_err()
        );
    }

    #[test]
    fn failed_placement_is_atomic_for_every_invalid_target() {
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Blocked;
        tiles[1] = Tile::Occupied(BoardCard {
            controller: BoardSide::Blue,
            card: card(2, 0, BattleClass::Physical, 0, 0, 0),
        });
        let mut board = Board::from_tiles(tiles);

        assert_eq!(
            board.place_card(
                card(3, 0, BattleClass::Physical, 0, 0, 0),
                Position::new(BOARD_SIZE, 0),
                BoardSide::Red,
            ),
            Err(PlacementError::OutOfBounds(Position::new(BOARD_SIZE, 0)))
        );
        assert_eq!(board.count_empty(), TILE_TOTAL - 2);

        assert_eq!(
            board.place_card(
                card(4, 0, BattleClass::Physical, 0, 0, 0),
                Position::new(0, 0),
                BoardSide::Red,
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
                BoardSide::Red,
            ),
            Err(PlacementError::Occupied(Position::new(0, 1)))
        );
        let occupied = board.get_card(Position::new(0, 1)).unwrap();
        assert_eq!(
            (occupied.controller, occupied.card.id),
            (BoardSide::Blue, 2)
        );
    }
}
