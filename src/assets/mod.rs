use std::sync::LazyLock;

use crate::{assets::card_record::CardRecord, models::card_asset::CardAsset};

pub mod card_record;

const CSV_FILE: &str = include_str!("./card_records.csv");

fn load_cards() -> Result<Vec<CardAsset>, String> {
    let mut reader = csv::Reader::from_reader(CSV_FILE.as_bytes());
    reader
        .deserialize::<CardRecord>()
        .enumerate()
        .map(|(index, res)| {
            let record = res.map_err(|e| e.to_string())?;
            CardAsset::try_from((index, record))
        })
        .collect()
}

pub static CARDS: LazyLock<Vec<CardAsset>> =
    LazyLock::new(|| load_cards().expect("failed to load cards"));
