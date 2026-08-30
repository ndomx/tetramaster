use std::fmt::Display;

use crate::models::{
    card_asset::CardAsset,
    direction::Direction::{self},
};

#[derive(PartialEq, Eq)]
pub struct Card {
    pub id: usize,
    pub directions: u8,
    pub asset: &'static CardAsset,
}

impl Card {
    pub fn new(id: usize, directions: u8, asset: &'static CardAsset) -> Self {
        Self {
            id,
            directions,
            asset,
        }
    }

    pub fn facing(&self) -> Vec<Direction> {
        return (0..8)
            .map(|offset| {
                let mask: u8 = 1 << offset;
                if self.directions & mask == 0 {
                    return None;
                }

                return match Direction::try_from(offset) {
                    Ok(value) => Some(value),
                    _ => None,
                };
            })
            .filter(|dir_opt| dir_opt.is_some())
            .map(|dir_opt| dir_opt.unwrap())
            .collect();
    }

    pub fn is_facing(&self, direction: &Direction) -> bool {
        let idx = *direction as u8;
        let mask: u8 = 1 << idx;
        return self.directions & mask > 0;
    }

    fn stats(&self) -> String {
        let atk = self.asset.attack >> 8;
        let phd = self.asset.phys_defense >> 8;
        let mgd = self.asset.mag_defense >> 8;

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
            self.directions
        )
    }
}
