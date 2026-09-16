use crate::models::card::Card;

#[derive(PartialEq, Eq)]
pub struct TileCard {
    pub owner_id: u64,
    pub card: Card,
}
