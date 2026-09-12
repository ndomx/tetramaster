use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS,
    models::{
        action::Action, active_player::ActivePlayer, board::Board, card::Card,
        game_state::GameState, player::Player,
    },
    utils::{constants::MAX_HAND_CARDS, random::VecRandomExt},
};

type TurnResult = Result<(), String>;

pub struct Game<'a> {
    pub board: Board,
    pub player: Player,
    pub cpu: Player,
    pub state: GameState,
    pub active_player: ActivePlayer,
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
            active_player: ActivePlayer::None,
        }
    }

    pub fn player_hand(&self) -> &Vec<Card> {
        return &self.player.hand;
    }

    pub fn run(&mut self) -> TurnResult {
        match self.state {
            GameState::NotStarted => self.start_game(),
            GameState::StartTurn => self.start_turn(),
            GameState::ApplyEffects => self.apply_effects(),
            GameState::EndTurn => self.end_turn(),
            _ => Ok(()),
        }
    }

    pub fn play_card(&mut self, action: Action) -> TurnResult {
        self.player_turn(action)?;

        self.state = GameState::ApplyEffects;
        Ok(())
    }

    pub fn awaiting_input(&self) -> bool {
        self.state == GameState::StartTurn && self.active_player == ActivePlayer::Player
    }

    fn start_turn(&mut self) -> TurnResult {
        if self.active_player != ActivePlayer::Cpu {
            return Err("Active player should be CPU".to_string());
        }

        self.cpu_turn()?;

        self.state = GameState::ApplyEffects;
        Ok(())
    }

    fn apply_effects(&mut self) -> TurnResult {
        self.state = GameState::EndTurn;
        Ok(())
    }

    fn end_turn(&mut self) -> TurnResult {
        // if game.finished

        self.active_player = match self.active_player {
            ActivePlayer::Cpu => ActivePlayer::Player,
            ActivePlayer::Player => ActivePlayer::Cpu,
            _ => return Err("Invalid active player".to_string()),
        };

        self.state = GameState::StartTurn;

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

    fn player_turn(&mut self, action: Action) -> TurnResult {
        if !self.board.is_available(action.target) {
            return Err("position is not available".to_string());
        }

        let player = &mut self.player;
        let Some(card) = player.pop_card(action.card_id) else {
            return Err("card id not found".to_string());
        };

        self.board.place_card(card, action.target, player.id)?;

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

        self.board.place_card(card, target, cpu.id)
    }

    fn start_game(&mut self) -> TurnResult {
        self.active_player = match self.rng.random_bool(0.5) {
            true => ActivePlayer::Player,
            false => ActivePlayer::Cpu,
        };

        self.state = GameState::StartTurn;

        Ok(())
    }
}
