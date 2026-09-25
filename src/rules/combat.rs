use crate::{
    models::core::card::{BattleClass, Card},
    utils::random::GameRng,
};

pub struct CombatParams<'a> {
    pub attacker: &'a Card,
    pub defender: &'a Card,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatOutcome {
    Victory,
    Defeat,
}

pub fn resolve_combat(params: CombatParams<'_>, rng: &mut GameRng) -> CombatOutcome {
    let attack_power = attack_value(params.attacker, rng);
    let defense_power = defense_value(params.defender, params.attacker.stats.battle_class, rng);

    resolve_power(attack_power, defense_power)
}

fn attack_value(card: &Card, rng: &mut GameRng) -> u8 {
    let power = boosted_power(card.stats.attack, rng.u8_below(16));
    let penalty = rng.u8_inclusive(power);

    power.saturating_sub(penalty)
}

fn defense_value(card: &Card, battle_class: BattleClass, rng: &mut GameRng) -> u8 {
    let stat = defense_stat(card, battle_class);
    let power = boosted_power(stat, rng.u8_below(16));
    let penalty = rng.u8_inclusive(power);

    power.saturating_sub(penalty)
}

fn boosted_power(stat: u8, bonus: u8) -> u8 {
    stat.saturating_add(bonus)
}

fn defense_stat(card: &Card, battle_class: BattleClass) -> u8 {
    match battle_class {
        BattleClass::Physical => card.stats.phys_defense,
        BattleClass::Magic => card.stats.mag_defense,
        BattleClass::Flexible => card.stats.phys_defense.min(card.stats.mag_defense),
        BattleClass::Assault => card
            .stats
            .phys_defense
            .min(card.stats.mag_defense)
            .min(card.stats.attack),
    }
}

fn resolve_power(attack_power: u8, defense_power: u8) -> CombatOutcome {
    if attack_power > defense_power {
        CombatOutcome::Victory
    } else {
        CombatOutcome::Defeat
    }
}

#[cfg(test)]
fn resolve_with_rolls(
    params: CombatParams<'_>,
    attack_bonus: u8,
    attack_penalty: u8,
    defense_bonus: u8,
    defense_penalty: u8,
) -> CombatOutcome {
    let attack_power = boosted_power(params.attacker.stats.attack, attack_bonus);
    let defense_power = boosted_power(
        defense_stat(params.defender, params.attacker.stats.battle_class),
        defense_bonus,
    );

    resolve_power(
        attack_power.saturating_sub(attack_penalty),
        defense_power.saturating_sub(defense_penalty),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;

    #[test]
    fn each_battle_class_selects_the_current_defense_stat() {
        let defender = card(2, 0, BattleClass::Physical, 5, 30, 20);
        let cases = [
            (BattleClass::Physical, 30),
            (BattleClass::Magic, 20),
            (BattleClass::Flexible, 20),
            (BattleClass::Assault, 5),
        ];

        for (class, selected_defense) in cases {
            let tied = card(1, 0, class, selected_defense, 0, 0);
            assert_eq!(
                resolve_with_rolls(
                    CombatParams {
                        attacker: &tied,
                        defender: &defender,
                    },
                    0,
                    0,
                    0,
                    0,
                ),
                CombatOutcome::Defeat
            );

            let winner = card(1, 0, class, selected_defense + 1, 0, 0);
            assert_eq!(
                resolve_with_rolls(
                    CombatParams {
                        attacker: &winner,
                        defender: &defender,
                    },
                    0,
                    0,
                    0,
                    0,
                ),
                CombatOutcome::Victory
            );
        }
    }

    #[test]
    fn deterministic_rolls_cover_victory_defeat_and_tie_as_defeat() {
        let attacker = card(1, 0, BattleClass::Physical, 20, 0, 0);
        let defender = card(2, 0, BattleClass::Physical, 0, 10, 0);
        let params = || CombatParams {
            attacker: &attacker,
            defender: &defender,
        };

        assert_eq!(
            resolve_with_rolls(params(), 0, 0, 0, 0),
            CombatOutcome::Victory
        );
        assert_eq!(
            resolve_with_rolls(params(), 0, 15, 0, 0),
            CombatOutcome::Defeat
        );
        assert_eq!(
            resolve_with_rolls(params(), 0, 10, 0, 0),
            CombatOutcome::Defeat
        );
    }

    #[test]
    fn combat_bonus_saturates_at_the_u8_upper_bound() {
        assert_eq!(boosted_power(u8::MAX, 15), u8::MAX);
        assert_eq!(boosted_power(u8::MAX - 5, 15), u8::MAX);
        assert_eq!(boosted_power(u8::MAX - 15, 15), u8::MAX);
    }
}
