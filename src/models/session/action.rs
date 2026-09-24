use crate::models::core::geometry::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    PlayCard { card_id: u64, position: Position },
}

impl GameAction {
    pub fn new(card_id: u64, target: Position) -> Self {
        Self::PlayCard {
            card_id,
            position: target,
        }
    }

    pub fn card_id(self) -> u64 {
        match self {
            Self::PlayCard { card_id, .. } => card_id,
        }
    }

    pub fn position(self) -> Position {
        match self {
            Self::PlayCard { position, .. } => position,
        }
    }
}
