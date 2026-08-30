use std::array;

use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS, constants::{MAX_HAND_CARDS, PLAYER_COUNT}, models::{board::Board, card::Card, player::Player},
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
            let name = match i {
                0 => String::from("Player"),
                _ => String::from("CPU"),
            };

            Game::build_player(name, rng)
        });

        let playing_idx = rng.random_bool(0.5) as usize;

        Self {
            board,
            players,
            playing_idx,
        }
    }

    fn build_player(name: String, rng: &mut ThreadRng) -> Player {
        let id: u64 = rng.next_u64();
        let hand = Game::build_hand(rng);

        Player { id, name, hand }
    }

    fn build_hand(rng: &mut ThreadRng) -> Vec<Card> {
        CARDS.sample(rng, MAX_HAND_CARDS).map(|asset| Card::new(asset)).collect()
    }
}
