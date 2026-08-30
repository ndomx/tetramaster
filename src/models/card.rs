use std::fmt::Display;

use crate::models::{
    battle_class::BattleClass,
    direction::Direction::{self},
};

#[derive(PartialEq, Eq)]
pub struct Card {
    pub id: usize,
    pub name: String,
    pub directions: u8,
    pub attack: u8,
    pub phys_defense: u8,
    pub mag_defense: u8,
    pub battle_class: BattleClass,
}

impl Card {
    pub fn new(
        id: usize,
        name: String,
        directions: u8,
        attack: u8,
        phys_defense: u8,
        mag_defense: u8,
        battle_class: BattleClass,
    ) -> Self {
        Self {
            id,
            name,
            directions,
            attack,
            phys_defense,
            mag_defense,
            battle_class,
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
        let atk = self.attack >> 8;
        let phd = self.phys_defense >> 8;
        let mgd = self.mag_defense >> 8;

        format!("{:X}{}{:X}{:X}", atk, self.battle_class, phd, mgd)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {:#X}",
            self.name,
            self.stats(),
            self.directions
        )
    }
}
