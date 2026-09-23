use std::sync::LazyLock;

use crate::{
    assets::card_record::CardRecord,
    models::core::card::{BattleClass, CardDefinition, CardStats},
};

pub mod card_record;

const CSV_FILE: &str = include_str!("./card_records.csv");

fn parse_stat(value: Option<char>) -> Result<u8, String> {
    let value = value.ok_or("unable to read char")?;
    let parsed = value
        .to_digit(16)
        .ok_or(format!("Invalid stat value {value}"))?;
    Ok(((parsed << 4) + 0xf) as u8)
}

fn parse_battle_class(value: Option<char>) -> Result<BattleClass, String> {
    match value.ok_or("unable to read char")? {
        'P' => Ok(BattleClass::Physical),
        'M' => Ok(BattleClass::Magic),
        'X' => Ok(BattleClass::Flexible),
        'A' => Ok(BattleClass::Assault),
        value => Err(format!("Invalid battle class:  {value}")),
    }
}

fn parse_definition(index: usize, record: CardRecord) -> Result<CardDefinition, String> {
    let mut stats = record.stats.chars();
    Ok(CardDefinition {
        index,
        name: record.name,
        base_stats: CardStats {
            attack: parse_stat(stats.next())?,
            battle_class: parse_battle_class(stats.next())?,
            phys_defense: parse_stat(stats.next())?,
            mag_defense: parse_stat(stats.next())?,
        },
    })
}

fn load_cards() -> Result<Vec<CardDefinition>, String> {
    let mut reader = csv::Reader::from_reader(CSV_FILE.as_bytes());
    reader
        .deserialize::<CardRecord>()
        .enumerate()
        .map(|(index, res)| {
            let record = res.map_err(|e| e.to_string())?;
            parse_definition(index, record)
        })
        .collect()
}

pub static CARDS: LazyLock<Vec<CardDefinition>> =
    LazyLock::new(|| load_cards().expect("failed to load cards"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_loads_every_current_record_with_required_fields() {
        assert_eq!(CARDS.len(), 100);
        for (index, card) in CARDS.iter().enumerate() {
            assert_eq!(card.index, index);
            assert!(!card.name.trim().is_empty());
            assert_eq!(card.base_stats.attack & 0x0f, 0x0f);
            assert_eq!(card.base_stats.phys_defense & 0x0f, 0x0f);
            assert_eq!(card.base_stats.mag_defense & 0x0f, 0x0f);
        }
    }

    fn record(stats: &str) -> CardRecord {
        CardRecord {
            name: "Test".into(),
            stats: stats.into(),
        }
    }

    #[test]
    fn parses_hex_stats_and_each_battle_class() {
        for (letter, class) in [
            ('P', BattleClass::Physical),
            ('M', BattleClass::Magic),
            ('X', BattleClass::Flexible),
            ('A', BattleClass::Assault),
        ] {
            let definition = parse_definition(3, record(&format!("A{letter}3F"))).unwrap();
            assert_eq!(definition.index, 3);
            assert_eq!(definition.base_stats.attack, 0xaf);
            assert_eq!(definition.base_stats.battle_class, class);
            assert_eq!(definition.base_stats.phys_defense, 0x3f);
            assert_eq!(definition.base_stats.mag_defense, 0xff);
        }
    }

    #[test]
    fn rejects_missing_invalid_or_unknown_stat_fields() {
        assert!(parse_definition(0, record("0P0")).is_err());
        assert!(parse_definition(0, record("GP00")).is_err());
        assert!(parse_definition(0, record("0Q00")).is_err());
    }
}
