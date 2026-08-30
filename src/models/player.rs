use crate::models::card::Card;

#[derive(PartialEq, Eq)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub hand: Vec<Card>,
}
