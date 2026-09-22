use crate::models::board_card::BoardCard;

#[derive(PartialEq, Eq)]
pub enum Tile {
    Empty,
    Blocked,
    Occupied(BoardCard),
}
