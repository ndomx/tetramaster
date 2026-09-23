use std::collections::VecDeque;

use super::PendingEffect;

#[derive(PartialEq, Debug)]
pub enum GamePhase {
    NotStarted,
    StartTurn,
    ApplyEffects { pending: VecDeque<PendingEffect> },
    EndTurn,
    Finished,
}
