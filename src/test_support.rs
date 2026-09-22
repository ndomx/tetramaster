use std::sync::LazyLock;

use crate::models::{
    battle_class::BattleClass, card::Card, card_asset::CardAsset, card_stats::CardStats, tile::Tile,
};

static PHYSICAL: LazyLock<CardAsset> = LazyLock::new(|| asset(BattleClass::Physical));
static MAGIC: LazyLock<CardAsset> = LazyLock::new(|| asset(BattleClass::Magic));
static FLEXIBLE: LazyLock<CardAsset> = LazyLock::new(|| asset(BattleClass::Flexible));
static ASSAULT: LazyLock<CardAsset> = LazyLock::new(|| asset(BattleClass::Assault));

fn asset(battle_class: BattleClass) -> CardAsset {
    CardAsset {
        index: 0,
        name: format!("{battle_class}"),
        base_stats: CardStats {
            attack: 0,
            battle_class,
            phys_defense: 0,
            mag_defense: 0,
        },
    }
}

pub fn card(
    id: u64,
    arrows: u8,
    battle_class: BattleClass,
    attack: u8,
    phys_defense: u8,
    mag_defense: u8,
) -> Card {
    let asset = match battle_class {
        BattleClass::Physical => &*PHYSICAL,
        BattleClass::Magic => &*MAGIC,
        BattleClass::Flexible => &*FLEXIBLE,
        BattleClass::Assault => &*ASSAULT,
    };

    Card {
        id,
        arrows,
        asset,
        stats: CardStats {
            attack,
            battle_class,
            phys_defense,
            mag_defense,
        },
    }
}

pub fn empty_tiles() -> [Tile; 16] {
    std::array::from_fn(|_| Tile::Empty)
}
