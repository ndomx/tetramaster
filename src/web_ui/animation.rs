use crate::models::session::{ControlChangeReason, GameEvent, PlayerSide};

use super::animation_constants::*;

pub fn presentation_duration(event: &GameEvent, reduced_motion: bool) -> u32 {
    if reduced_motion {
        return REDUCED_MOTION_MS;
    }

    match event {
        GameEvent::GameStarted { .. } => GAME_STARTED_MS,
        GameEvent::TurnStarted {
            player: PlayerSide::Human,
        } => HUMAN_TURN_STARTED_MS,
        GameEvent::TurnStarted {
            player: PlayerSide::Cpu,
        } => CPU_TURN_STARTED_MS,
        GameEvent::CardPlaced { .. } => CARD_PLACED_MS,
        GameEvent::CombatResolved { .. } => COMBAT_RESOLVED_MS,
        GameEvent::ControlChanged {
            reason: ControlChangeReason::Combo,
            ..
        } => COMBO_CONTROL_CHANGED_MS,
        GameEvent::ControlChanged { .. } => CONTROL_CHANGED_MS,
        GameEvent::TurnEnded { .. } => TURN_ENDED_MS,
        GameEvent::GameFinished { .. } => GAME_FINISHED_MS,
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn wait_for_presentation(duration_ms: u32) {
    gloo_timers::future::TimeoutFuture::new(duration_ms).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn wait_for_presentation(_duration_ms: u32) {}

#[cfg(target_arch = "wasm32")]
pub fn system_prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media("(prefers-reduced-motion: reduce)").ok())
        .flatten()
        .is_some_and(|query| query.matches())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn system_prefers_reduced_motion() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{core::geometry::Position, session::GameResult};

    #[test]
    fn durations_are_centralized_by_event_and_motion_preference() {
        assert_eq!(
            presentation_duration(
                &GameEvent::TurnStarted {
                    player: PlayerSide::Cpu,
                },
                false,
            ),
            CPU_TURN_STARTED_MS
        );
        assert_eq!(
            presentation_duration(
                &GameEvent::CardPlaced {
                    player: PlayerSide::Human,
                    card_id: 1,
                    position: Position::new(0, 0),
                },
                false,
            ),
            CARD_PLACED_MS
        );
        assert_eq!(
            presentation_duration(
                &GameEvent::GameFinished {
                    result: GameResult::Draw,
                },
                true,
            ),
            REDUCED_MOTION_MS
        );
    }
}
