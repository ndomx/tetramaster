use std::{collections::HashSet, fmt, sync::LazyLock};

use serde::Deserialize;

use crate::models::core::card::{BattleClass, CardArtwork, CardDefinition, CardStats};

pub mod artwork;

const CATALOG_FILE: &str = include_str!("./card_catalog.ron");
const FALLBACK_PATH: &str = "assets/cards/fallback.png";
const FALLBACK_IMAGE: &[u8] = include_bytes!("../../assets/cards/fallback.png");

#[derive(Debug, Deserialize)]
struct CatalogRecord {
    artwork: ArtworkDefaults,
    cards: Vec<CardRecord>,
}

#[derive(Debug, Deserialize)]
struct ArtworkDefaults {
    width: u16,
    height: u16,
    fallback_path: String,
}

#[derive(Debug, Deserialize)]
struct CardRecord {
    name: String,
    attack: u8,
    battle_class: BattleClassRecord,
    physical_defense: u8,
    magical_defense: u8,
    artwork_filename: String,
}

#[derive(Debug, Deserialize)]
enum BattleClassRecord {
    Physical,
    Magic,
    Flexible,
    Assault,
}

impl From<BattleClassRecord> for BattleClass {
    fn from(value: BattleClassRecord) -> Self {
        match value {
            BattleClassRecord::Physical => Self::Physical,
            BattleClassRecord::Magic => Self::Magic,
            BattleClassRecord::Flexible => Self::Flexible,
            BattleClassRecord::Assault => Self::Assault,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CatalogError(String);

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn load_cards(source: &str) -> Result<Vec<CardDefinition>, CatalogError> {
    let catalog: CatalogRecord = ron::from_str(source)
        .map_err(|error| CatalogError(format!("catalog syntax error: {error}")))?;

    validate_artwork_defaults(&catalog.artwork)?;

    let mut names = HashSet::with_capacity(catalog.cards.len());
    let mut artwork_filenames = HashSet::with_capacity(catalog.cards.len());
    catalog
        .cards
        .into_iter()
        .enumerate()
        .map(|(index, record)| {
            validate_record(index, &record, &mut names, &mut artwork_filenames)?;
            Ok(CardDefinition {
                index,
                name: record.name,
                base_stats: CardStats {
                    attack: record.attack,
                    battle_class: record.battle_class.into(),
                    phys_defense: record.physical_defense,
                    mag_defense: record.magical_defense,
                },
                artwork: CardArtwork {
                    filename: record.artwork_filename,
                    fallback_path: catalog.artwork.fallback_path.clone(),
                    width: catalog.artwork.width,
                    height: catalog.artwork.height,
                },
            })
        })
        .collect()
}

fn validate_artwork_defaults(artwork: &ArtworkDefaults) -> Result<(), CatalogError> {
    if artwork.width != 84 || artwork.height != 102 {
        return Err(CatalogError(format!(
            "catalog artwork dimensions must be 84x102, got {}x{}",
            artwork.width, artwork.height
        )));
    }
    if artwork.fallback_path != FALLBACK_PATH {
        return Err(CatalogError(format!(
            "catalog artwork fallback_path must be {FALLBACK_PATH:?}, got {:?}",
            artwork.fallback_path
        )));
    }
    let fallback_dimensions = png_dimensions(FALLBACK_IMAGE).ok_or_else(|| {
        CatalogError(format!(
            "catalog artwork fallback {FALLBACK_PATH:?} must be a valid PNG"
        ))
    })?;
    if fallback_dimensions != (u32::from(artwork.width), u32::from(artwork.height)) {
        return Err(CatalogError(format!(
            "catalog artwork fallback {FALLBACK_PATH:?} dimensions must be {}x{}, got {}x{}",
            artwork.width, artwork.height, fallback_dimensions.0, fallback_dimensions.1
        )));
    }
    Ok(())
}

pub(crate) fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

fn validate_record(
    index: usize,
    record: &CardRecord,
    names: &mut HashSet<String>,
    artwork_filenames: &mut HashSet<String>,
) -> Result<(), CatalogError> {
    let entry = format!("catalog entry {} ({:?})", index + 1, record.name);
    if record.name.trim().is_empty() {
        return Err(CatalogError(format!("{entry}: name must not be empty")));
    }
    if !names.insert(record.name.clone()) {
        return Err(CatalogError(format!("{entry}: duplicate name")));
    }
    for (field, value) in [
        ("attack", record.attack),
        ("physical_defense", record.physical_defense),
        ("magical_defense", record.magical_defense),
    ] {
        if value & 0x0f != 0x0f {
            return Err(CatalogError(format!(
                "{entry}: {field} must be an expanded hexadecimal tier ending in 15, got {value}"
            )));
        }
    }
    if !artwork::is_valid_artwork_filename(&record.artwork_filename) {
        return Err(CatalogError(format!(
            "{entry}: artwork_filename must be a snake_case PNG filename, got {:?}",
            record.artwork_filename
        )));
    }
    if !artwork_filenames.insert(record.artwork_filename.clone()) {
        return Err(CatalogError(format!("{entry}: duplicate artwork_filename")));
    }
    Ok(())
}

pub static CARDS: LazyLock<Vec<CardDefinition>> =
    LazyLock::new(|| load_cards(CATALOG_FILE).expect("failed to load embedded card catalog"));

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
            assert_eq!(card.artwork.width, 84);
            assert_eq!(card.artwork.height, 102);
            assert_eq!(card.artwork.fallback_path, "assets/cards/fallback.png");
            assert!(artwork::is_valid_artwork_filename(&card.artwork.filename));
        }
    }

    #[test]
    fn ron_uses_typed_battle_classes() {
        for (source, expected) in [
            ("Physical", BattleClass::Physical),
            ("Magic", BattleClass::Magic),
            ("Flexible", BattleClass::Flexible),
            ("Assault", BattleClass::Assault),
        ] {
            let record: BattleClassRecord = ron::from_str(source).unwrap();
            assert_eq!(BattleClass::from(record), expected);
        }
    }

    #[test]
    fn invalid_entries_report_their_position_and_name() {
        let source = r#"(
            artwork: (width: 84, height: 102, fallback_path: "assets/cards/fallback.png"),
            cards: [(name: "Broken", attack: 16, battle_class: Physical,
                physical_defense: 15, magical_defense: 15,
                artwork_filename: "broken.png")],
        )"#;

        let error = load_cards(source).unwrap_err().to_string();
        assert!(error.contains("catalog entry 1 (\"Broken\")"), "{error}");
        assert!(error.contains("attack"), "{error}");
    }

    #[test]
    fn duplicate_artwork_filenames_are_rejected() {
        let source = r#"(
            artwork: (width: 84, height: 102, fallback_path: "assets/cards/fallback.png"),
            cards: [
                (name: "First", attack: 15, battle_class: Physical,
                    physical_defense: 15, magical_defense: 15,
                    artwork_filename: "shared.png"),
                (name: "Second", attack: 15, battle_class: Physical,
                    physical_defense: 15, magical_defense: 15,
                    artwork_filename: "shared.png"),
            ],
        )"#;

        let error = load_cards(source).unwrap_err().to_string();
        assert!(error.contains("duplicate artwork_filename"), "{error}");
    }
}
