use super::BoardCard;

#[derive(PartialEq, Eq)]
pub enum Tile {
    Empty,
    Blocked,
    Occupied(BoardCard),
}
