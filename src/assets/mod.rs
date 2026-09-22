use std::sync::LazyLock;

use crate::{assets::card_record::CardRecord, models::card_definition::CardDefinition};

pub mod card_record;

const CSV_FILE: &str = include_str!("./card_records.csv");

fn load_cards() -> Result<Vec<CardDefinition>, String> {
    let mut reader = csv::Reader::from_reader(CSV_FILE.as_bytes());
    reader
        .deserialize::<CardRecord>()
        .enumerate()
        .map(|(index, res)| {
            let record = res.map_err(|e| e.to_string())?;
            CardDefinition::try_from((index, record))
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
}
