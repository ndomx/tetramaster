use std::fmt::Display;

use rand::random;

use crate::models::{
    card_asset::CardAsset, card_stats::CardStats, direction::Direction::{self},
};

#[derive(PartialEq, Eq)]
pub struct Card {
    pub id: u64,
    pub arrows: u8,
    pub asset: &'static CardAsset,
    pub stats: CardStats,
}

impl Card {
    pub fn new(asset: &'static CardAsset) -> Self {
        let arrows = random();
        let id = random();

        let stats = CardStats::generate(&asset.base_stats);

        Self { id, arrows, asset, stats }
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
