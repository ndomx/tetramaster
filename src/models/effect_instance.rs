use crate::models::effect::Effect;

#[derive(PartialEq, Debug)]
pub struct EffectInstance {
    pub source_card_id: u64,
    pub target_card_id: u64,
    pub effect: Effect,
}

impl EffectInstance {
    pub fn new(source_card_id: u64, target_card_id: u64, effect: Effect) -> Self {
        Self {
            source_card_id,
            target_card_id,
            effect,
        }
    }
}
