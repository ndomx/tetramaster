use std::fmt::{Display, Formatter};

use crate::{
    models::core::{board::Board, geometry::Position},
    utils::{constants::TILE_TOTAL, helpers::idx2pos},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlacementInteractionKind {
    Battle,
    DirectCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementInteraction {
    pub source_card_id: u64,
    pub target_card_id: u64,
    pub kind: PlacementInteractionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementInteractionError {
    pub position: Position,
}

impl Display for PlacementInteractionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "could not find placed card at {:?}",
            self.position
        )
    }
}

impl std::error::Error for PlacementInteractionError {}

pub fn is_legal_position(board: &Board, position: Position) -> bool {
    board.is_available(position)
}

pub fn legal_positions(board: &Board) -> Vec<Position> {
    (0..TILE_TOTAL)
        .filter_map(idx2pos)
        .filter(|position| is_legal_position(board, *position))
        .collect()
}

pub fn discover_interactions(
    board: &Board,
    position: Position,
) -> Result<Vec<PlacementInteraction>, PlacementInteractionError> {
    let placed = board
        .get_card(position)
        .ok_or(PlacementInteractionError { position })?;

    let mut interactions: Vec<_> = placed
        .card
        .facing()
        .into_iter()
        .filter_map(|direction| {
            board
                .get_relative(position, &direction)
                .map(|neighbor| (neighbor, direction))
        })
        .filter(|(neighbor, _)| neighbor.owner_id != placed.owner_id)
        .map(|(neighbor, direction)| PlacementInteraction {
            source_card_id: placed.card.id,
            target_card_id: neighbor.card.id,
            kind: if neighbor.card.is_facing(direction.opposite()) {
                PlacementInteractionKind::Battle
            } else {
                PlacementInteractionKind::DirectCapture
            },
        })
        .collect();

    interactions.sort_by_key(|interaction| interaction.kind);
    Ok(interactions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::core::{
            board::{BoardCard, Tile},
            card::BattleClass,
            geometry::Direction,
        },
        test_support::{card, empty_tiles},
        utils::{constants::BOARD_SIZE, helpers::pos2idx},
    };

    fn occupied(owner_id: u64, id: u64, arrows: u8) -> Tile {
        Tile::Occupied(BoardCard {
            owner_id,
            card: card(id, arrows, BattleClass::Physical, 0, 0, 0),
        })
    }

    #[test]
    fn legal_positions_include_only_empty_in_bounds_tiles_in_board_order() {
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Blocked;
        tiles[1] = occupied(1, 1, 0);
        let board = Board::from_tiles(tiles);

        let positions = legal_positions(&board);

        assert_eq!(positions.len(), 14);
        assert_eq!(positions[0], Position::new(0, 2));
        assert_eq!(positions[13], Position::new(3, 3));
        assert!(!is_legal_position(&board, Position::new(0, 0)));
        assert!(!is_legal_position(&board, Position::new(0, 1)));
        assert!(!is_legal_position(&board, Position::new(BOARD_SIZE, 0)));
    }

    #[test]
    fn interactions_ignore_allies_and_unpointed_neighbors() {
        let mut tiles = empty_tiles();
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 10, 1 << Direction::E as u8);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(1, 20, 0);
        tiles[pos2idx(Position::new(0, 1)).unwrap()] = occupied(2, 30, 0);

        assert!(
            discover_interactions(&Board::from_tiles(tiles), Position::new(1, 1))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn battles_precede_direct_captures_while_preserving_direction_order() {
        let mut tiles = empty_tiles();
        let arrows =
            (1 << Direction::N as u8) | (1 << Direction::E as u8) | (1 << Direction::S as u8);
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 10, arrows);
        tiles[pos2idx(Position::new(0, 1)).unwrap()] = occupied(2, 20, 1 << Direction::S as u8);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(2, 30, 0);
        tiles[pos2idx(Position::new(2, 1)).unwrap()] = occupied(2, 40, 1 << Direction::N as u8);

        assert_eq!(
            discover_interactions(&Board::from_tiles(tiles), Position::new(1, 1)).unwrap(),
            vec![
                PlacementInteraction {
                    source_card_id: 10,
                    target_card_id: 20,
                    kind: PlacementInteractionKind::Battle,
                },
                PlacementInteraction {
                    source_card_id: 10,
                    target_card_id: 40,
                    kind: PlacementInteractionKind::Battle,
                },
                PlacementInteraction {
                    source_card_id: 10,
                    target_card_id: 30,
                    kind: PlacementInteractionKind::DirectCapture,
                },
            ]
        );
    }

    #[test]
    fn interaction_discovery_requires_a_placed_card() {
        let position = Position::new(0, 0);
        assert_eq!(
            discover_interactions(&Board::from_tiles(empty_tiles()), position),
            Err(PlacementInteractionError { position })
        );
    }
}
