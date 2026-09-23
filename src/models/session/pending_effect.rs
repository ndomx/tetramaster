use super::Effect;

#[derive(PartialEq, Debug)]
pub struct PendingEffect {
    pub source_card_id: u64,
    pub target_card_id: u64,
    pub effect: Effect,
}

impl PendingEffect {
    pub fn new(source_card_id: u64, target_card_id: u64, effect: Effect) -> Self {
        Self {
            source_card_id,
            target_card_id,
            effect,
        }
    }
}
