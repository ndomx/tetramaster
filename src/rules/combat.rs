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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatResolution {
    pub outcome: CombatOutcome,
    pub attack_power: u8,
    pub defense_power: u8,
}

pub fn resolve_combat(params: CombatParams<'_>, rng: &mut GameRng) -> CombatResolution {
    let attack_power = attack_value(params.attacker, rng);
    let defense_power = defense_value(params.defender, params.attacker.stats.battle_class, rng);

    CombatResolution {
        outcome: resolve_power(attack_power, defense_power),
        attack_power,
        defense_power,
    }
}

fn attack_value(card: &Card, rng: &mut GameRng) -> u8 {
    let power = boosted_power(attack_stat(card), rng.u8_below(16));
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

fn attack_stat(card: &Card) -> u8 {
    match card.stats.battle_class {
        BattleClass::Assault => card
            .stats
            .attack
            .max(card.stats.phys_defense)
            .max(card.stats.mag_defense),
        _ => card.stats.attack,
    }
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
    let attack_power = boosted_power(attack_stat(params.attacker), attack_bonus);
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
    fn assault_uses_the_challengers_highest_stat_and_opponents_lowest_stat() {
        for (attack, physical, magic) in [(30, 20, 10), (10, 30, 20), (20, 10, 30)] {
            let attacker = card(1, 0, BattleClass::Assault, attack, physical, magic);
            assert_eq!(attack_stat(&attacker), 30);

            for (attack, physical, magic) in [(5, 40, 50), (40, 5, 50), (40, 50, 5)] {
                let defender = card(2, 0, BattleClass::Physical, attack, physical, magic);
                assert_eq!(defense_stat(&defender, BattleClass::Assault), 5);
                assert_eq!(
                    resolve_with_rolls(
                        CombatParams {
                            attacker: &attacker,
                            defender: &defender
                        },
                        0,
                        24,
                        0,
                        0,
                    ),
                    CombatOutcome::Victory
                );
            }
        }
    }

    #[test]
    fn other_classes_use_only_the_challengers_attack_stat() {
        for class in [
            BattleClass::Physical,
            BattleClass::Magic,
            BattleClass::Flexible,
        ] {
            let attacker = card(1, 0, class, 10, 30, 50);
            assert_eq!(attack_stat(&attacker), 10);
        }
    }

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

    #[test]
    fn resolution_reports_the_exact_powers_used_for_the_outcome() {
        let attacker = card(1, 0, BattleClass::Physical, 20, 0, 0);
        let defender = card(2, 0, BattleClass::Physical, 0, 10, 0);
        let resolution = resolve_combat(
            CombatParams {
                attacker: &attacker,
                defender: &defender,
            },
            &mut GameRng::from_seed(42),
        );

        assert_eq!(
            resolution.outcome,
            resolve_power(resolution.attack_power, resolution.defense_power)
        );
    }
}
