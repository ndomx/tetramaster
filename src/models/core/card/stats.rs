use super::BattleClass;
use crate::utils::random::GameRng;

#[derive(PartialEq, Eq, Debug)]
pub struct CardStats {
    pub attack: u8,
    pub battle_class: BattleClass,
    pub phys_defense: u8,
    pub mag_defense: u8,
}

impl CardStats {
    pub fn generate(base_stats: &Self, rng: &mut GameRng) -> Self {
        let attack = rng.u8_inclusive(base_stats.attack);
        let phys_defense = rng.u8_inclusive(base_stats.phys_defense);
        let mag_defense = rng.u8_inclusive(base_stats.mag_defense);

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
    fn generated_stats_preserve_class_and_include_the_base_bounds() {
        const SEED: u64 = 0x51A7;
        let base = CardStats {
            attack: 1,
            battle_class: BattleClass::Magic,
            phys_defense: 1,
            mag_defense: 1,
        };
        let mut rng = GameRng::from_seed(SEED);
        let mut saw_upper_bound = [false; 3];
        for _ in 0..100 {
            let generated = CardStats::generate(&base, &mut rng);
            assert_eq!(generated.battle_class, BattleClass::Magic);
            assert!(generated.attack <= base.attack);
            assert!(generated.phys_defense <= base.phys_defense);
            assert!(generated.mag_defense <= base.mag_defense);
            saw_upper_bound[0] |= generated.attack == base.attack;
            saw_upper_bound[1] |= generated.phys_defense == base.phys_defense;
            saw_upper_bound[2] |= generated.mag_defense == base.mag_defense;
        }
        assert!(
            saw_upper_bound.into_iter().all(|seen| seen),
            "seed {SEED} never generated one or more inclusive upper bounds"
        );
    }

    #[test]
    fn zero_base_stats_generate_zero_without_panicking() {
        const SEED: u64 = 0;
        let base = CardStats {
            attack: 0,
            battle_class: BattleClass::Physical,
            phys_defense: 0,
            mag_defense: 0,
        };
        let mut rng = GameRng::from_seed(SEED);

        let generated = CardStats::generate(&base, &mut rng);

        assert_eq!(generated.attack, 0, "seed {SEED}");
        assert_eq!(generated.phys_defense, 0, "seed {SEED}");
        assert_eq!(generated.mag_defense, 0, "seed {SEED}");
    }
}
