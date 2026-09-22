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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::battle_class::BattleClass, test_support::card};

    #[test]
    fn pop_card_removes_only_the_first_matching_identity() {
        let mut player = Player {
            id: 1,
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
