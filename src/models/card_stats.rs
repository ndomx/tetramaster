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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_stats_preserve_class_and_stay_below_current_exclusive_bounds() {
        let base = CardStats {
            attack: 16,
            battle_class: BattleClass::Magic,
            phys_defense: 32,
            mag_defense: 48,
        };
        for _ in 0..100 {
            let generated = CardStats::generate(&base);
            assert_eq!(generated.battle_class, BattleClass::Magic);
            assert!(generated.attack < base.attack);
            assert!(generated.phys_defense < base.phys_defense);
            assert!(generated.mag_defense < base.mag_defense);
        }
    }
}
