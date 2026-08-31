use crate::{
    models::position::Position,
    utils::constants::{BOARD_SIZE, TILE_TOTAL},
};

pub fn idx2pos(idx: usize) -> Option<Position> {
    if idx >= TILE_TOTAL {
        return None;
    }

    let row = idx / BOARD_SIZE;
    let col = idx % BOARD_SIZE;

    Some(Position { row, col })
}

pub fn pos2idx(pos: Position) -> Option<usize> {
    if pos.row >= BOARD_SIZE || pos.col >= BOARD_SIZE {
        return None;
    }

    Some(pos.row * BOARD_SIZE + pos.col)
}
