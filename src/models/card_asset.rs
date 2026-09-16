use crate::models::card_stats::CardStats;

#[derive(PartialEq, Eq)]
pub struct CardAsset {
    pub index: usize,
    pub name: &'static str,
    pub base_stats: CardStats,
}
