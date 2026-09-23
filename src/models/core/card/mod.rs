use std::fmt::Display;

mod battle_class;
mod definition;
mod stats;

pub use battle_class::BattleClass;
pub use definition::CardDefinition;
pub use stats::CardStats;

use crate::{models::core::geometry::Direction, utils::random::GameRng};

#[derive(PartialEq, Eq)]
pub struct Card {
    pub id: u64,
    pub arrows: u8,
    pub asset: &'static CardDefinition,
    pub stats: CardStats,
}

impl Card {
    pub fn new(asset: &'static CardDefinition, rng: &mut GameRng) -> Self {
        let arrows = rng.next_u8();
        let id = rng.next_u64();

        let stats = CardStats::generate(&asset.base_stats, rng);

        Self {
            id,
            arrows,
            asset,
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
        let idx = direction as u8;
        let mask: u8 = 1 << idx;

        self.arrows & mask > 0
    }

    pub fn format_stats(&self) -> String {
        let atk = self.stats.attack >> 4;
        let phd = self.stats.phys_defense >> 4;
        let mgd = self.stats.mag_defense >> 4;

        format!("{:X}{}{:X}{:X}", atk, self.stats.battle_class, phd, mgd)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {:#X}",
            self.asset.name,
            self.format_stats(),
            self.arrows
        )
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

    #[test]
    fn displayed_stats_use_high_nibbles_and_class_letter() {
        let card = card(1, 0, BattleClass::Flexible, 0xaf, 0x31, 0x09);
        assert_eq!(card.format_stats(), "AX30");
    }
}
