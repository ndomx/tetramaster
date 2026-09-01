use std::fmt::Display;

use rand::random;

use crate::models::{
    card_asset::CardAsset,
    direction::Direction::{self},
};

#[derive(PartialEq, Eq)]
pub struct Card {
    pub id: u64,
    pub arrows: u8,
    pub asset: &'static CardAsset,
}

impl Card {
    pub fn new(asset: &'static CardAsset) -> Self {
        let arrows = random();
        let id = random();

        Self { id, arrows, asset }
    }

    pub fn facing(&self) -> Vec<Direction> {
        return (0..8)
            .filter_map(|offset| {
                let mask: u8 = 1 << offset;
                if self.arrows & mask == 0 {
                    return None;
                }

                match Direction::try_from(offset) {
                    Ok(value) => Some(value),
                    _ => None,
                }
            })
            .collect();
    }

    pub fn is_facing(&self, direction: &Direction) -> bool {
        let idx = *direction as u8;
        let mask: u8 = 1 << idx;
        return self.arrows & mask > 0;
    }

    pub fn stats(&self) -> String {
        let atk = self.asset.attack >> 4;
        let phd = self.asset.phys_defense >> 4;
        let mgd = self.asset.mag_defense >> 4;

        format!("{:X}{}{:X}{:X}", atk, self.asset.battle_class, phd, mgd)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {:#X}",
            self.asset.name,
            self.stats(),
            self.arrows
        )
    }
}
