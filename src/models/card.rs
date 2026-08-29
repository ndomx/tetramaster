use std::fmt::Display;

use crate::models::{
    attack_type::AttackType,
    direction::Direction::{self},
};

#[derive(PartialEq, Eq)]
pub struct Card {
    pub name: String,
    pub directions: u8,
    pub attack: u8,
    pub phys_defense: u8,
    pub mag_defense: u8,
    pub attack_type: AttackType,
}

impl Card {
    pub fn new(
        name: String,
        directions: u8,
        attack: u8,
        phys_defense: u8,
        mag_defense: u8,
        attack_type: AttackType,
    ) -> Self {
        Self {
            name,
            directions,
            attack,
            phys_defense,
            mag_defense,
            attack_type,
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
        format!(
            "{:X}{}{:X}{:X}",
            self.attack, self.attack_type, self.phys_defense, self.mag_defense
        )
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
