#[derive(PartialEq, Debug)]
pub enum GameState {
    NotStarted,
    StartTurn,
    ApplyEffects,
    EndTurn,
    Finished,
}
