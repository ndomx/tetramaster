use crate::models::{card::Card, player::Player};

#[derive(PartialEq, Eq)]
pub struct PlayedCard {
    pub owner_id: usize,
    pub card_id: usize,
}