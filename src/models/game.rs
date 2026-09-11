use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS,
    models::{action::Action, board::Board, card::Card, game_state::GameState, player::Player},
    utils::{constants::MAX_HAND_CARDS, random::VecRandomExt},
};

type TurnResult = Result<(), String>;

pub struct Game<'a> {
    pub board: Board,
    pub player: Player,
    pub cpu: Player,
    pub state: GameState,
    rng: &'a mut ThreadRng,
}

impl<'a> Game<'a> {
    pub fn new(block_density: f64, rng: &'a mut ThreadRng) -> Self {
        let board = Board::build(block_density, rng);

        let player = Game::build_player(false, rng);
        let cpu = Game::build_player(true, rng);

        Self {
            board,
            player,
            cpu,
            rng,
            state: GameState::NotStarted,
        }
    }

    pub fn player_hand(&self) -> &Vec<Card> {
        return &self.player.hand;
    }

    pub fn run(&mut self) -> TurnResult {
        match self.state {
            GameState::NotStarted => self.start_game(),
            GameState::CpuTurn => self.cpu_turn(),
            _ => Ok(()),
        }
    }

    pub fn play_card(&mut self, action: Action) -> TurnResult {
        self.handle_turn(action)?;
        self.end_turn()
    }

    fn end_turn(&mut self) -> TurnResult {
        // if game.finished

        self.state = match self.state {
            GameState::CpuTurn => Ok(GameState::AwaitingPlayer),
            GameState::AwaitingPlayer => Ok(GameState::CpuTurn),
            _ => Err(format!("Cannot end turn from state {:?}", self.state)),
        }?;

        Ok(())
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

    fn handle_turn(&mut self, action: Action) -> TurnResult {
        if !self.board.is_available(action.target) {
            return Err("position is not available".to_string());
        }

        let player = &mut self.player;
        let Some(card) = player.pop_card(action.card_id) else {
            return Err("card id not found".to_string());
        };

        self.board.place_card(card, action.target, player.id)?;

        // let _neighbouring_enemies =
        //     self.board
        //         .neighboring_enemies(target, card.facing(), player.id);

        // attack

        Ok(())
    }

    fn cpu_turn(&mut self) -> TurnResult {
        let Some(target) = self.board.find_available(self.rng) else {
            return Err("unable to find a position".to_string());
        };

        let cpu = &mut self.cpu;
        let Some(card) = cpu.hand.take_random(self.rng) else {
            return Err("unable to draw a card from cpu".to_string());
        };

        self.board.place_card(card, target, cpu.id)?;

        // let _neighbouring_enemies =
        //     self.board
        //         .neighboring_enemies(target, card.facing(), player.id);

        // attack

        self.end_turn()
    }

    fn start_game(&mut self) -> TurnResult {
        self.state = match self.rng.random_bool(0.5) {
            true => GameState::AwaitingPlayer,
            false => GameState::CpuTurn,
        };

        Ok(())
    }
}
