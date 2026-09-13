use crate::models::{card::Card, position::Position};

pub struct PlacedCard {
    pub card: Card,
    pub pos: Position,
}

impl PlacedCard {
    pub fn new(card: Card, pos: Position) -> Self {
        Self { card, pos }
    }
}
