use std::array;

use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS,
    models::{board::Board, card::Card, player::Player, position::Position, tile::Tile},
    utils::constants::{MAX_HAND_CARDS, PLAYER_COUNT},
};

pub struct Game {
    pub board: Board,
    pub players: [Player; PLAYER_COUNT],
    playing_idx: usize,
}

impl Game {
    pub fn new(block_density: f64, rng: &mut ThreadRng) -> Self {
        let board = Board::build(block_density, rng);

        let players: [Player; 2] = array::from_fn(|i| {
            let is_cpu = i > 0;
            Game::build_player(is_cpu, rng)
        });

        let playing_idx = rng.random_bool(0.5) as usize;

        Self {
            board,
            players,
            playing_idx,
        }
    }

    fn build_player(is_cpu: bool, rng: &mut ThreadRng) -> Player {
        let id: u64 = rng.next_u64();
        let hand = Game::build_hand(rng);

        Player { id, is_cpu, hand }
    }

    fn build_hand(rng: &mut ThreadRng) -> Vec<Card> {
        CARDS
            .sample(rng, MAX_HAND_CARDS)
            .map(|asset| Card::new(asset))
            .collect()
    }

    fn handle_turn(&mut self, card_id: u64, target: Position) -> Result<(), ()> {
        if !self.board.is_available(target) {
            return Err(());
        }

        let player = &mut self.players[self.playing_idx];

        let card_opt = player.pop_card(card_id);
        if card_opt.is_none() {
            return Err(());
        }

        let card = card_opt.unwrap();
        self.board.place_card(card.id, target, player.id)?;

        let neighbouring_enemies = self
            .board
            .neighboring_enemies(target, card.facing(), player.id);

        // attack

        Ok(())
    }
}
