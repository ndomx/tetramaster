use crate::{
    min,
    models::{battle_class::BattleClass, card::Card},
};

pub struct AttackParams<'a> {
    pub attacker: &'a Card,
    pub defender: &'a Card,
}

pub enum AttackOutcome {
    Victory,
    Defeat,
}

type AttackResult = Result<AttackOutcome, String>;

pub fn attack<'a>(params: AttackParams<'a>) -> AttackResult {
    let attacker = params.attacker;
    let attack_power = attack_value(attacker);

    let defender = params.defender;
    let defense_power = defense_value(defender, attacker.stats.battle_class);

    println!("atk={}, def={}", attack_power, defense_power);
    match attack_power > defense_power {
        true => Ok(AttackOutcome::Victory),
        false => Ok(AttackOutcome::Defeat),
    }
}

fn attack_value(card: &Card) -> u8 {
    let atk_pwr = card.stats.attack + rand::random_range(0..16u8);
    let atk_penalty = rand::random_range(0..=atk_pwr);

    atk_pwr.saturating_sub(atk_penalty)
}

fn defense_value(card: &Card, battle_class: BattleClass) -> u8 {
    let def_stat = defense_stat(card, battle_class);

    let def_pwr = def_stat + rand::random_range(0..16u8);
    let def_penalty = rand::random_range(0..=def_pwr);

    def_pwr.saturating_sub(def_penalty)
}

fn defense_stat(card: &Card, battle_class: BattleClass) -> u8 {
    match battle_class {
        BattleClass::Physical => card.stats.phys_defense,
        BattleClass::Magic => card.stats.mag_defense,
        BattleClass::Flexible => {
            min!(card.stats.phys_defense, card.stats.mag_defense)
        }
        BattleClass::Assault => min!(
            card.stats.phys_defense,
            card.stats.mag_defense,
            card.stats.attack
        ),
    }
}

#[cfg(test)]
fn attack_with_rolls(
    params: AttackParams<'_>,
    attack_bonus: u8,
    attack_penalty: u8,
    defense_bonus: u8,
    defense_penalty: u8,
) -> AttackOutcome {
    let attack_power = params.attacker.stats.attack + attack_bonus;
    let defense_stat = defense_stat(params.defender, params.attacker.stats.battle_class);
    let defense_power = defense_stat + defense_bonus;
    if attack_power.saturating_sub(attack_penalty) > defense_power.saturating_sub(defense_penalty) {
        AttackOutcome::Victory
    } else {
        AttackOutcome::Defeat
    }
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
            assert!(matches!(
                attack_with_rolls(
                    AttackParams {
                        attacker: &tied,
                        defender: &defender
                    },
                    0,
                    0,
                    0,
                    0
                ),
                AttackOutcome::Defeat
            ));
            let winner = card(1, 0, class, selected_defense + 1, 0, 0);
            assert!(matches!(
                attack_with_rolls(
                    AttackParams {
                        attacker: &winner,
                        defender: &defender
                    },
                    0,
                    0,
                    0,
                    0
                ),
                AttackOutcome::Victory
            ));
        }
    }

    #[test]
    fn deterministic_roll_hook_covers_victory_defeat_and_tie_as_defeat() {
        let attacker = card(1, 0, BattleClass::Physical, 20, 0, 0);
        let defender = card(2, 0, BattleClass::Physical, 0, 10, 0);
        assert!(matches!(
            attack_with_rolls(
                AttackParams {
                    attacker: &attacker,
                    defender: &defender
                },
                0,
                0,
                0,
                0
            ),
            AttackOutcome::Victory
        ));
        assert!(matches!(
            attack_with_rolls(
                AttackParams {
                    attacker: &attacker,
                    defender: &defender
                },
                0,
                15,
                0,
                0
            ),
            AttackOutcome::Defeat
        ));
        assert!(matches!(
            attack_with_rolls(
                AttackParams {
                    attacker: &attacker,
                    defender: &defender
                },
                0,
                10,
                0,
                0
            ),
            AttackOutcome::Defeat
        ));
    }
}
