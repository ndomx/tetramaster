use std::collections::VecDeque;

use crate::models::{
    board::Board, direction::Direction, effect::Effect, effect_instance::EffectInstance,
    position::Position, tile_card::TileCard,
};

pub struct GenerateEffectsParams<'a> {
    pub position: Position,
    pub board: &'a Board,
}

pub fn generate_effects<'a>(
    params: GenerateEffectsParams<'a>,
) -> Result<VecDeque<EffectInstance>, String> {
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

    let mut effects: Vec<EffectInstance> = neighbors
        .iter()
        .map(|&(defender, dir)| {
            let e = match defender.card.is_facing(dir.opposite()) {
                true => Effect::Attack,
                false => Effect::Capture,
            };

            EffectInstance::new(tc_ref.card.id, defender.card.id, e)
        })
        .collect();

    effects.sort_by_key(|ef_instance| ef_instance.effect.priority());
    Ok(VecDeque::from(effects))
}

fn scan_neighbors(
    board: &Board,
    ally_id: u64,
    position: Position,
    dirs: Vec<Direction>,
) -> Vec<(&TileCard, Direction)> {
    dirs.iter()
        .filter_map(|dir| board.get_relative(position, dir).map(|tc| (tc, *dir)))
        .filter(|(tc, _)| tc.owner_id != ally_id)
        .collect()
}
