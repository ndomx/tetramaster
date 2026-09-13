use crate::models::{tile_card::TileCard};

#[derive(PartialEq, Eq)]
pub enum Tile {
    Empty,
    Block,
    Card(TileCard),
}
