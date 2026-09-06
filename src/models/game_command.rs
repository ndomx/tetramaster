use crate::models::position::Position;

pub enum GameTurnOutput {
    RenderBoard,
    RenderHand,
    SelectPosition
}

pub enum GameState {
    NotStarted,
    CpuTurnStart,
    CpuTurnEnd,
    PlayerTurnStart,
    PlayerTurnEnd,
}

pub enum GameTurnInput {
    Continue,
    PlaceCard { card_id: u64, target: Position },
}
