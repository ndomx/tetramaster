#[derive(PartialEq)]
pub enum GameState {
    NotStarted,
    CpuTurnStart,
    CpuTurnEnd,
    PlayerTurnStart,
    PlayerTurnEnd,
}
