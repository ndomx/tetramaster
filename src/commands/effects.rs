use std::collections::VecDeque;

use crate::models::{
    core::{
        board::{Board, BoardCard},
        geometry::{Direction, Position},
    },
    session::{Effect, PendingEffect},
};
use crate::rules::placement::{PlacementInteractionKind, discover_interactions};

pub struct GenerateEffectsParams<'a> {
    pub position: Position,
    pub board: &'a Board,
}

pub fn generate_effects<'a>(
    params: GenerateEffectsParams<'a>,
) -> Result<VecDeque<PendingEffect>, String> {
    Ok(discover_interactions(params.board, params.position)
        .map_err(|_| "Could not find card".to_string())?
        .into_iter()
        .map(|interaction| {
            let effect = match interaction.kind {
                PlacementInteractionKind::Battle => Effect::Attack,
                PlacementInteractionKind::DirectCapture => Effect::DirectCapture,
            };
            PendingEffect::new(
                interaction.source_card_id,
                interaction.target_card_id,
                effect,
            )
        })
        .collect())
}

pub fn spread_victory_effects<'a>(
    params: GenerateEffectsParams<'a>,
    pending_effects: &'a mut VecDeque<PendingEffect>,
) -> Result<(), String> {
    let tc_ref = params
        .board
        .get_card(params.position)
        .ok_or("Could not find card")?;

    let neighbors = scan_neighbors(
        params.board,
        tc_ref.owner_id,
        params.position,
        tc_ref.card.facing(),
    );

    neighbors.iter().for_each(|&(defender, _)| {
        let effect = PendingEffect::new(tc_ref.card.id, defender.card.id, Effect::DirectCapture);
        pending_effects.retain(|eff| eff.target_card_id != defender.card.id);
        pending_effects.push_front(effect);
    });

    Ok(())
}

fn scan_neighbors(
    board: &Board,
    ally_id: u64,
    position: Position,
    dirs: Vec<Direction>,
) -> Vec<(&BoardCard, Direction)> {
    dirs.iter()
        .filter_map(|dir| board.get_relative(position, dir).map(|tc| (tc, *dir)))
        .filter(|(tc, _)| tc.owner_id != ally_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{
            core::{
                board::{Board, BoardCard, Tile},
                card::BattleClass,
            },
            session::Effect,
        },
        test_support::{card, empty_tiles},
        utils::helpers::pos2idx,
    };

    fn occupied(owner_id: u64, id: u64, arrows: u8) -> Tile {
        Tile::Occupied(BoardCard {
            owner_id,
            card: card(id, arrows, BattleClass::Physical, 0, 0, 0),
        })
    }

    #[test]
    fn no_arrows_or_only_allied_neighbors_generate_no_effects() {
        let mut tiles = empty_tiles();
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 1, 0);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(2, 2, 0);
        assert!(
            generate_effects(GenerateEffectsParams {
                position: Position::new(1, 1),
                board: &Board::from_tiles(tiles)
            })
            .unwrap()
            .is_empty()
        );

        let mut tiles = empty_tiles();
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 1, 1 << Direction::E as u8);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(1, 2, 0);
        assert!(
            generate_effects(GenerateEffectsParams {
                position: Position::new(1, 1),
                board: &Board::from_tiles(tiles)
            })
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn direct_capture_and_opposed_arrow_attack_are_prioritized() {
        let mut tiles = empty_tiles();
        let center_arrows = (1 << Direction::N as u8) | (1 << Direction::E as u8);
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 10, center_arrows);
        tiles[pos2idx(Position::new(0, 1)).unwrap()] = occupied(2, 20, 0);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(2, 30, 1 << Direction::W as u8);
        let effects = generate_effects(GenerateEffectsParams {
            position: Position::new(1, 1),
            board: &Board::from_tiles(tiles),
        })
        .unwrap();
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].target_card_id, 30);
        assert_eq!(effects[0].effect, Effect::Attack);
        assert_eq!(effects[1].target_card_id, 20);
        assert_eq!(effects[1].effect, Effect::DirectCapture);
    }

    #[test]
    fn victory_spread_prepends_captures_and_removes_existing_target_effects() {
        let mut tiles = empty_tiles();
        tiles[pos2idx(Position::new(1, 1)).unwrap()] = occupied(1, 10, 1 << Direction::E as u8);
        tiles[pos2idx(Position::new(1, 2)).unwrap()] = occupied(2, 20, 1 << Direction::W as u8);
        let board = Board::from_tiles(tiles);
        let mut pending = VecDeque::from([
            PendingEffect::new(99, 20, Effect::Attack),
            PendingEffect::new(99, 30, Effect::DirectCapture),
        ]);
        spread_victory_effects(
            GenerateEffectsParams {
                position: Position::new(1, 1),
                board: &board,
            },
            &mut pending,
        )
        .unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(
            (
                pending[0].source_card_id,
                pending[0].target_card_id,
                &pending[0].effect
            ),
            (10, 20, &Effect::DirectCapture)
        );
        assert_eq!(pending[1].target_card_id, 30);
    }
}
