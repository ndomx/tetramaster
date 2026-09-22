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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_and_indices_round_trip_across_the_board() {
        for index in 0..TILE_TOTAL {
            let position = idx2pos(index).unwrap();
            assert_eq!(pos2idx(position), Some(index));
        }
        assert!(idx2pos(TILE_TOTAL).is_none());
        assert!(pos2idx(Position::new(BOARD_SIZE, 0)).is_none());
        assert!(pos2idx(Position::new(0, BOARD_SIZE)).is_none());
    }
}
