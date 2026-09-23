use std::fmt::{Display, Formatter};

use crate::models::core::{board::Board, geometry::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboCapture {
    pub source_card_id: u64,
    pub target_card_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboError {
    pub position: Position,
}

impl Display for ComboError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "could not find combo source card at {:?}",
            self.position
        )
    }
}

impl std::error::Error for ComboError {}

/// Returns the current engine's one-hop combo captures in processing order.
///
/// The original game presents these captures simultaneously. Session
/// orchestration currently applies each returned capture as an individual effect.
pub fn discover_combo_captures(
    board: &Board,
    position: Position,
) -> Result<Vec<ComboCapture>, ComboError> {
    let source = board.get_card(position).ok_or(ComboError { position })?;

    let mut captures: Vec<_> = source
        .card
        .facing()
        .into_iter()
        .filter_map(|direction| board.get_relative(position, &direction))
        .filter(|neighbor| neighbor.owner_id != source.owner_id)
        .map(|neighbor| ComboCapture {
            source_card_id: source.card.id,
            target_card_id: neighbor.card.id,
        })
        .collect();

    captures.reverse();
    Ok(captures)
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
        utils::helpers::pos2idx,
    };

    fn occupied(owner_id: u64, id: u64, arrows: u8) -> Tile {
        Tile::Occupied(BoardCard {
            owner_id,
            card: card(id, arrows, BattleClass::Physical, 0, 0, 0),
        })
    }

    #[test]
    fn combo_discovery_is_one_hop_and_preserves_current_reverse_direction_order() {
        let mut tiles = empty_tiles();
        let arrows =
            (1 << Direction::N as u8) | (1 << Direction::E as u8) | (1 << Direction::S as u8);
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 10, arrows);
        tiles[pos2idx(Position::new(0, 1)).unwrap()] = occupied(2, 20, 0);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(1, 30, 0);
        tiles[pos2idx(Position::new(2, 1)).unwrap()] = occupied(2, 40, 0);

        assert_eq!(
            discover_combo_captures(&Board::from_tiles(tiles), Position::new(1, 1)).unwrap(),
            vec![
                ComboCapture {
                    source_card_id: 10,
                    target_card_id: 40,
                },
                ComboCapture {
                    source_card_id: 10,
                    target_card_id: 20,
                },
            ]
        );
    }

    #[test]
    fn combo_discovery_requires_a_source_card() {
        let position = Position::new(0, 0);
        assert_eq!(
            discover_combo_captures(&Board::from_tiles(empty_tiles()), position),
            Err(ComboError { position })
        );
    }
}
