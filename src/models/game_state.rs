#[derive(PartialEq, Debug)]
pub enum GameState {
    NotStarted,
    AwaitingPlayer,
    CpuTurn,
    Finished,
}
