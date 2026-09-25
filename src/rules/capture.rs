use std::fmt::{Display, Formatter};

use crate::models::core::{
    board::{Board, BoardSide},
    geometry::Position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    Direct,
    CombatVictory,
    CombatDefeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capture {
    pub card_id: u64,
    pub new_controller: BoardSide,
    pub kind: CaptureKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureOutcome {
    pub position: Position,
    pub previous_controller: BoardSide,
    pub capture: Capture,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    CardNotFound(u64),
    ControlChange(String),
}

impl Display for CaptureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardNotFound(card_id) => write!(formatter, "card {card_id} was not found"),
            Self::ControlChange(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CaptureError {}

pub fn apply_capture(board: &mut Board, capture: Capture) -> Result<CaptureOutcome, CaptureError> {
    let (board_card, position) = board
        .position_of_card(capture.card_id)
        .ok_or(CaptureError::CardNotFound(capture.card_id))?;
    let previous_controller = board_card.controller;

    board
        .set_controller(position, capture.new_controller)
        .map_err(CaptureError::ControlChange)?;

    Ok(CaptureOutcome {
        position,
        previous_controller,
        capture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::core::{
            board::{BoardCard, Tile},
            card::BattleClass,
        },
        test_support::{card, empty_tiles},
    };

    #[test]
    fn capture_changes_only_the_requested_cards_controller() {
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Occupied(BoardCard {
            controller: BoardSide::Blue,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            controller: BoardSide::Red,
            card: card(20, 0, BattleClass::Physical, 0, 0, 0),
        });
        let mut board = Board::from_tiles(tiles);
        let capture = Capture {
            card_id: 20,
            new_controller: BoardSide::Blue,
            kind: CaptureKind::Direct,
        };

        let outcome = apply_capture(&mut board, capture).unwrap();

        assert_eq!(outcome.position, Position::new(0, 1));
        assert_eq!(outcome.previous_controller, BoardSide::Red);
        assert_eq!(outcome.capture, capture);
        assert_eq!(
            board.get_card(Position::new(0, 0)).unwrap().controller,
            BoardSide::Blue
        );
        assert_eq!(
            board.get_card(Position::new(0, 1)).unwrap().controller,
            BoardSide::Blue
        );
    }

    #[test]
    fn missing_card_fails_without_mutating_the_board() {
        let mut board = Board::from_tiles(empty_tiles());

        assert_eq!(
            apply_capture(
                &mut board,
                Capture {
                    card_id: 99,
                    new_controller: BoardSide::Blue,
                    kind: CaptureKind::CombatVictory,
                }
            ),
            Err(CaptureError::CardNotFound(99))
        );
        assert_eq!(board.count_empty(), 16);
    }
}
