use super::CardStats;

#[derive(PartialEq, Eq)]
pub struct CardDefinition {
    pub index: usize,
    pub name: String,
    pub base_stats: CardStats,
}
