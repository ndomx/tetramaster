use crate::models::core::{board::BoardSide, card::Card};

#[derive(PartialEq, Eq)]
pub struct BoardCard {
    pub controller: BoardSide,
    pub card: Card,
}
