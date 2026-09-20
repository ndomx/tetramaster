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
    CardAsset {
        index: 9,
        name: "Ironite",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Physical,
            phys_defense: 0x1f,
            mag_defense: 0xf,
        },
    },
    CardAsset {
        index: 10,
        name: "Sahagin",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Physical,
            phys_defense: 0x1f,
            mag_defense: 0x0f,
        },
    },
    CardAsset {
        index: 11,
        name: "Yeti",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Magic,
            phys_defense: 0xf,
            mag_defense: 0x1f,
        },
    },
    CardAsset {
        index: 12,
        name: "Mimic",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x1f,
            mag_defense: 0x1f,
        },
    },
    CardAsset {
        index: 13,
        name: "Wyerd",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x0f,
            mag_defense: 0x1f,
        },
    },
    CardAsset {
        index: 14,
        name: "Mandragora",
        base_stats: CardStats {
            attack: 0x1f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x0f,
            mag_defense: 0x2f,
        },
    },
    CardAsset {
        index: 15,
        name: "Crawler",
        base_stats: CardStats {
            attack: 0x2f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x2f,
            mag_defense: 0x0f,
        },
    },
    CardAsset {
        index: 16,
        name: "Sand Scorpion",
        base_stats: CardStats {
            attack: 0x2f,
            battle_class: BattleClass::Physical,
            phys_defense: 0x2f,
            mag_defense: 0x0f,
        },
    },
    CardAsset {
        index: 44,
        name: "Vepal",
        base_stats: CardStats {
            attack: 0x5f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x3f,
            mag_defense: 0x3f,
        },
    },
    CardAsset {
        index: 45,
        name: "Grimlock",
        base_stats: CardStats {
            attack: 0x5f,
            battle_class: BattleClass::Magic,
            phys_defense: 0x2f,
            mag_defense: 0x3f,
        },
    },
    CardAsset {
        index: 46,
        name: "Tonberry",
        base_stats: CardStats {
            attack: 0x2f,
            battle_class: BattleClass::Physical,
            phys_defense: 0x3f,
            mag_defense: 0x3f,
        },
    },
    CardAsset {
        index: 54,
        name: "Nova Dragon",
        base_stats: CardStats {
            attack: 0xef,
            battle_class: BattleClass::Physical,
            phys_defense: 0x7f,
            mag_defense: 0xcf,
        },
    },
    CardAsset {
        index: 55,
        name: "Ozma",
        base_stats: CardStats {
            attack: 0xdf,
            battle_class: BattleClass::Magic,
            phys_defense: 0x0f,
            mag_defense: 0xCf,
        },
    },
    CardAsset {
        index: 56,
        name: "Hades",
        base_stats: CardStats {
            attack: 0xff,
            battle_class: BattleClass::Magic,
            phys_defense: 0xcf,
            mag_defense: 0x1f,
        },
    },
    CardAsset {
        index: 64,
        name: "Odin",
        base_stats: CardStats {
            attack: 0xcf,
            battle_class: BattleClass::Magic,
            phys_defense: 0x8f,
            mag_defense: 0x4f,
        },
    },
    CardAsset {
        index: 65,
        name: "Leviathan",
        base_stats: CardStats {
            attack: 0xbf,
            battle_class: BattleClass::Magic,
            phys_defense: 0x6f,
            mag_defense: 0x1f,
        },
    },
    CardAsset {
        index: 66,
        name: "Bahamut",
        base_stats: CardStats {
            attack: 0xcf,
            battle_class: BattleClass::Magic,
            phys_defense: 0x8f,
            mag_defense: 0x5f,
        },
    },
];
