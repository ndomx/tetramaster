use crate::{
    models::core::geometry::Position,
    utils::constants::{BOARD_SIZE, TILE_TOTAL},
};

pub fn idx2pos(index: usize) -> Option<Position> {
    if index >= TILE_TOTAL {
        return None;
    }

    let row = index / BOARD_SIZE;
    let col = index % BOARD_SIZE;

    Some(Position { row, col })
}

pub fn pos2idx(position: Position) -> Option<usize> {
    if position.row >= BOARD_SIZE || position.col >= BOARD_SIZE {
        return None;
    }

    Some(position.row * BOARD_SIZE + position.col)
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
