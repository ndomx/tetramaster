use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CardRecord {
    pub name: String,
    pub stats: String,
}
