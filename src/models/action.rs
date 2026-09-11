use crate::models::position::Position;

pub struct Action {
    pub card_id: u64,
    pub target: Position,
}

impl Action {
    pub fn new(card_id: u64, target: Position) -> Self {
        Self { card_id, target }
    }
}
