use super::{board::BoardSide, card::Card};

#[derive(PartialEq, Eq)]
pub struct Player {
    pub board_side: BoardSide,
    pub name: String,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn pop_card(&mut self, card_id: u64) -> Option<Card> {
        self.hand
            .iter()
            .position(|card| card.id == card_id)
            .map(|index| self.hand.remove(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::core::card::BattleClass, test_support::card};

    #[test]
    fn pop_card_removes_only_the_first_matching_identity() {
        let mut player = Player {
            board_side: BoardSide::Blue,
            name: "P".into(),
            hand: vec![
                card(4, 0, BattleClass::Physical, 0, 0, 0),
                card(5, 0, BattleClass::Physical, 0, 0, 0),
            ],
        };
        assert_eq!(player.pop_card(5).unwrap().id, 5);
        assert_eq!(player.hand.len(), 1);
        assert!(player.pop_card(99).is_none());
        assert_eq!(player.hand[0].id, 4);
    }
}
