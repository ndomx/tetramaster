use std::collections::VecDeque;

use crate::models::effect_instance::EffectInstance;

#[derive(PartialEq, Debug)]
pub enum GameState {
    NotStarted,
    StartTurn,
    ApplyEffects { pending: VecDeque<EffectInstance> },
    EndTurn,
    Finished,
}
