use crate::models::battle_class::BattleClass;

#[derive(PartialEq, Eq)]
pub struct CardAsset {
    pub index: usize,
    pub name: &'static str,
    pub attack: u8,
    pub battle_class: BattleClass,
    pub phys_defense: u8,
    pub mag_defense: u8,
}