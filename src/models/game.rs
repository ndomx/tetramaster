use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS,
    models::{
        board::Board,
        card::Card,
        game_command::{GameState, GameTurnInput, GameTurnOutput},
        player::Player,
        position::Position,
    },
    utils::{
        constants::{MAX_HAND_CARDS},
        random::VecRandomExt,
    },
};

pub struct Game<'a> {
    pub board: Board,
    pub player: Player,
    pub cpu: Player,
    rng: &'a mut ThreadRng,
    state: GameState,
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

    fn handle_turn(&mut self, card_id: u64, target: Position) -> Result<(), String> {
        if !self.board.is_available(target) {
            return Err("position is not available".to_string());
        }

        let player = &mut self.player;
        let Some(card) = player.pop_card(card_id) else {
            return Err("card id not found".to_string());
        };

        self.board.place_card(card, target, player.id)?;

        // let _neighbouring_enemies =
        //     self.board
        //         .neighboring_enemies(target, card.facing(), player.id);

        // attack

        Ok(())
    }

    fn bot_turn(&mut self) -> Result<(), String> {
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

        Ok(())
    }

    fn start_game(&mut self, _input: GameTurnInput) -> GameTurnOutput {
        self.state = match self.rng.random_bool(0.5) {
            true => GameState::PlayerTurnStart,
            false => GameState::CpuTurnStart,
        };

        GameTurnOutput::RenderBoard
    }

    fn start_cpu_turn(&mut self, _input: GameTurnInput) -> GameTurnOutput {
        self.bot_turn().ok();
        self.state = GameState::CpuTurnEnd;

        GameTurnOutput::RenderBoard
    }

    fn end_cpu_turn(&mut self, _input: GameTurnInput) -> GameTurnOutput {
        self.state = GameState::PlayerTurnStart;

        GameTurnOutput::SelectPosition
    }

    fn start_player_turn(&mut self, input: GameTurnInput) -> GameTurnOutput {
        let GameTurnInput::PlaceCard { card_id, target } = input else {
            return GameTurnOutput::SelectPosition;
        };

        let res = self.handle_turn(card_id, target);
        if res.is_err() {
            return GameTurnOutput::SelectPosition;
        }

        self.state = GameState::PlayerTurnEnd;
        GameTurnOutput::RenderBoard
    }

    fn end_player_turn(&mut self, _input: GameTurnInput) -> GameTurnOutput {
        self.state = GameState::CpuTurnStart;
        GameTurnOutput::Continue
    }
}
