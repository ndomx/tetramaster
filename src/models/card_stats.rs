use rand::random_range;

use crate::models::battle_class::BattleClass;

#[derive(PartialEq, Eq, Debug)]
pub struct CardStats {
    pub attack: u8,
    pub battle_class: BattleClass,
    pub phys_defense: u8,
    pub mag_defense: u8,
}

impl CardStats {
    pub fn generate(base_stats: &Self) -> Self {
        let attack = random_range(0..base_stats.attack);
        let phys_defense = random_range(0..base_stats.phys_defense);
        let mag_defense = random_range(0..base_stats.mag_defense);

        Self {
            attack,
            phys_defense,
            mag_defense,
            battle_class: base_stats.battle_class,
        }
    }
}
