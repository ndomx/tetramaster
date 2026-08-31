use crate::models::{played_card::PlayedCard};

#[derive(PartialEq, Eq)]
pub enum Tile {
    Empty,
    Block,
    Card(PlayedCard),
}
