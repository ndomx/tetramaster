use crate::{
    min,
    models::{battle_class::BattleClass, card::Card},
};

pub struct AttackParams<'a> {
    pub attacker: &'a Card,
    pub defender: &'a Card,
}

pub enum AttackOutcome {
    Win,
    Lose,
}

type AttackResult = Result<AttackOutcome, String>;

pub fn attack<'a>(params: AttackParams<'a>) -> AttackResult {
    let attacker = params.attacker;
    let attack_power = attack_value(attacker);

    let defender = params.defender;
    let defense_power = defense_value(defender, attacker.stats.battle_class);

    println!("atk={}, def={}", attack_power, defense_power);
    match attack_power > defense_power {
        true => Ok(AttackOutcome::Win),
        false => Ok(AttackOutcome::Lose),
    }
}

fn attack_value(card: &Card) -> u8 {
    let atk_pwr = card.stats.attack + rand::random_range(0..16u8);
    let atk_penalty = rand::random_range(0..=atk_pwr);

    atk_pwr.saturating_sub(atk_penalty)
}

fn defense_value(card: &Card, battle_class: BattleClass) -> u8 {
    let def_stat = match battle_class {
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
    };

    let def_pwr = def_stat + rand::random_range(0..16u8);
    let def_penalty = rand::random_range(0..=def_pwr);

    def_pwr.saturating_sub(def_penalty)
}
