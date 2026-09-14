use crate::models::card::Card;

#[derive(PartialEq, Eq)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn pop_card(&mut self, card_id: u64) -> Option<Card> {
        self.hand
            .iter()
            .position(|c| c.id == card_id)
            .map(|idx| self.hand.remove(idx))
    }
}
