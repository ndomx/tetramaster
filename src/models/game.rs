use std::array;

use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS, models::{
        board::Board, card::Card, game_command::{GameState, GameTurnInput, GameTurnOutput}, player::Player, position::Position,
    }, utils::{constants::{MAX_HAND_CARDS, PLAYER_COUNT}, random::VecRandomExt},
};

pub struct Game<'a> {
    pub board: Board,
    pub players: [Player; PLAYER_COUNT],
    playing_idx: usize,
    rng: &'a mut ThreadRng,
    state: GameState,
}

impl<'a> Game<'a> {
    pub fn new(block_density: f64, rng: &'a mut ThreadRng) -> Self {
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
            rng,
            state: GameState::NotStarted,
        }
    }

    pub fn player_hand(&self) -> &Vec<Card> {
        return &self.players[0].hand;
    }

    pub fn run(&mut self, input: GameTurnInput) -> GameTurnOutput {
        match self.state {
            GameState::NotStarted => self.start_game(input),
            GameState::CpuTurnStart => self.start_cpu_turn(input),
            GameState::CpuTurnEnd => self.end_cpu_turn(input),
            GameState::PlayerTurnStart => self.start_player_turn(input),
            GameState::PlayerTurnEnd => self.end_player_turn(input),
        }
    }


    fn build_player(is_cpu: bool, rng: &mut ThreadRng) -> Player {
        let id: u64 = rng.next_u64();
        let hand = Game::build_hand(rng);
        let name = match is_cpu {
            true => "CPU".to_string(),
            false => "Player".to_string(),
        };

        Player { id, name, hand }
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
        let Some(card) = player.pop_card(card_id) else {
            return Err(());
        };

        self.board.place_card(card, target, player.id)?;

        // let _neighbouring_enemies =
        //     self.board
        //         .neighboring_enemies(target, card.facing(), player.id);

        // attack

        Ok(())
    }

    fn bot_turn(&mut self) -> Result<(), ()> {
        let Some(target) = self.board.find_available(self.rng) else {
            return Err(());
        };

        let player = &mut self.players[self.playing_idx];
        let Some(card) = player.hand.take_random(self.rng) else {
            return Err(());
        };

        self.board.place_card(card, target, player.id)?;

        // let _neighbouring_enemies =
        //     self.board
        //         .neighboring_enemies(target, card.facing(), player.id);

        // attack

        Ok(())
    }

    fn start_game(&mut self, _input: GameTurnInput) -> GameTurnOutput {
        self.state = match self.playing_idx {
            0 => GameState::PlayerTurnStart,
            _ => GameState::CpuTurnStart,
        };

        GameTurnOutput::RenderBoard
    }

    fn start_cpu_turn(&mut self, _input: GameTurnInput) -> GameTurnOutput {
        self.bot_turn().ok();
        self.state = GameState::CpuTurnEnd;

        GameTurnOutput::RenderBoard
    }

    fn end_cpu_turn(&mut self, input: GameTurnInput) -> GameTurnOutput {
        self.state = GameState::PlayerTurnStart;
        self.playing_idx = 0;

        GameTurnOutput::SelectPosition
    }

    fn start_player_turn(&mut self, input: GameTurnInput) -> GameTurnOutput {
        match input {
            GameTurnInput::Continue => GameTurnOutput::SelectPosition,
            GameTurnInput::PlaceCard { card_id, target } => {
                self.handle_turn(card_id, target).ok();
                GameTurnOutput::RenderBoard
            },
        }
    }

    fn end_player_turn(&mut self, input: GameTurnInput) -> GameTurnOutput {
        todo!()
    }
}
