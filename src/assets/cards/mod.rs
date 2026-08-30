use crate::models::{battle_class::BattleClass, card_asset::CardAsset};

pub const GOBLIN: CardAsset = CardAsset {
    index: 1,
    name: "Goblin",
    attack: 0,
    battle_class: BattleClass::Physical,
    phys_defense: 0,
    mag_defense: 0,
};

pub const FANG: CardAsset = CardAsset {
    index: 2,
    name: "Fang",
    attack: 0,
    battle_class: BattleClass::Physical,
    phys_defense: 0,
    mag_defense: 0,
};

pub const SKELETON: CardAsset = CardAsset {
    index: 3,
    name: "Skeleton",
    attack: 0,
    battle_class: BattleClass::Physical,
    phys_defense: 0,
    mag_defense: 0,
};

pub const FLAN: CardAsset = CardAsset {
    index: 4,
    name: "Flan",
    attack: 0,
    battle_class: BattleClass::Magic,
    phys_defense: 0,
    mag_defense: 16,
};
