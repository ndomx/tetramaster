mod battle_class;
mod definition;
mod stats;

pub use battle_class::BattleClass;
pub use definition::{CardArtwork, CardDefinition};
pub use stats::CardStats;

use crate::{models::core::geometry::Direction, utils::random::GameRng};

#[derive(PartialEq, Eq, Debug)]
pub struct Card {
    pub id: u64,
    pub arrows: u8,
    pub definition: &'static CardDefinition,
    pub stats: CardStats,
}

impl Card {
    pub fn new(definition: &'static CardDefinition, rng: &mut GameRng) -> Self {
        let arrows = rng.next_u8();
        let id = rng.next_u64();

        let stats = CardStats::generate(&definition.base_stats, rng);

        Self {
            id,
            arrows,
            definition,
            stats,
        }
    }

    pub fn facing(&self) -> Vec<Direction> {
        (0..8)
            .filter_map(|offset| {
                let mask: u8 = 1 << offset;
                if self.arrows & mask == 0 {
                    return None;
                }

                Direction::try_from(offset).ok()
            })
            .collect()
    }

    pub fn is_facing(&self, direction: Direction) -> bool {
        let index = direction as u8;
        let mask: u8 = 1 << index;

        self.arrows & mask > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;

    #[test]
    fn arrow_mask_detects_and_orders_all_facing_directions() {
        let card = card(1, 0b1010_0101, BattleClass::Physical, 0, 0, 0);
        assert_eq!(
            card.facing(),
            vec![Direction::N, Direction::E, Direction::SW, Direction::NW]
        );
        assert!(card.is_facing(Direction::SW));
        assert!(!card.is_facing(Direction::S));
    }
}
