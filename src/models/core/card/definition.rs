use super::CardStats;

#[derive(PartialEq, Eq, Debug)]
pub struct CardArtwork {
    pub source_url: String,
    pub fallback_path: String,
    pub width: u16,
    pub height: u16,
}

#[derive(PartialEq, Eq, Debug)]
pub struct CardDefinition {
    pub index: usize,
    pub name: String,
    pub base_stats: CardStats,
    pub artwork: CardArtwork,
}
