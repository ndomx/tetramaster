use crate::models::card::Card;

#[derive(PartialEq, Eq)]
pub enum Tile {
    Empty,
    Block,
    Card(Card),
}
