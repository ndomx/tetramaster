use crate::models::card::Card;

#[derive(PartialEq, Eq)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub hand: Vec<Card>,
}
