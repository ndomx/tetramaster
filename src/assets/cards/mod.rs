use crate::models::{battle_class::BattleClass, card_asset::CardAsset, card_stats::CardStats};

pub const CARDS: &[CardAsset] = &[
    CardAsset {
        index: 1,
        name: "Goblin",
        base_stats: CardStats {
            attack: 0xf,
            battle_class: BattleClass::Physical,
            phys_defense: 0xf,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 2,
        name: "Fang",
        base_stats: CardStats {
            attack: 0xf,
            battle_class: BattleClass::Physical,
            phys_defense: 0xf,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 3,
        name: "Skeleton",
        base_stats: CardStats {
            attack: 0xf,
            battle_class: BattleClass::Physical,
            phys_defense: 0xf,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 4,
        name: "Flan",
        base_stats: CardStats {
            attack: 0xf,
            battle_class: BattleClass::Magic,
            phys_defense: 0xf,
            mag_defense: 0x1f,
        },
    },
    CardAsset {
        index: 5,
        name: "Zaghnol",
        base_stats: CardStats {
            attack: 0xf,
            battle_class: BattleClass::Physical,
            phys_defense: 0xf,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 6,
        name: "Lizard Man",
        base_stats: CardStats {
            attack: 0xf,
            battle_class: BattleClass::Physical,
            phys_defense: 0xf,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 7,
        name: "Zombie",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x1f,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 8,
        name: "Bomb",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Magic,
            phys_defense: 0xf,
            mag_defense: 0x1f,
        },
    },
];
