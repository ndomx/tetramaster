use super::{board::BoardSide, card::Card};

#[derive(PartialEq, Eq)]
pub struct Player {
    pub board_side: BoardSide,
    pub hand: Vec<Card>,
}
