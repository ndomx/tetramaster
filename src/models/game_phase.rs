use std::collections::VecDeque;

use crate::models::pending_effect::PendingEffect;

#[derive(PartialEq, Debug)]
pub enum GamePhase {
    NotStarted,
    StartTurn,
    ApplyEffects { pending: VecDeque<PendingEffect> },
    EndTurn,
    Finished,
}
