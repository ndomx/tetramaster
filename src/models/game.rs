use crate::models::{board::Board, player::Player};

pub struct Game {
    pub board: Board,
    pub players: [Player; 2],
    playing_idx: usize,
}
