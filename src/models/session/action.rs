use crate::models::core::geometry::Position;

pub struct GameAction {
    pub card_id: u64,
    pub target: Position,
}

impl GameAction {
    pub fn new(card_id: u64, target: Position) -> Self {
        Self { card_id, target }
    }
}
