use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS,
    commands::{AttackOutcome, AttackParams, GenerateEffectsParams, attack, generate_effects},
    models::{
        action::Action, active_player::ActivePlayer, board::Board, card::Card,
        effect_instance::EffectInstance, game_state::GameState, player::Player, position::Position,
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
            // placed_cards: vec![],
        }
    }

    pub fn player_hand(&self) -> &Vec<Card> {
        &self.player.hand
    }

    pub fn run(&mut self) -> TurnResult {
        match self.state {
            GameState::NotStarted => self.start_game(),
            GameState::StartTurn => self.start_turn(),
            GameState::ApplyEffects { pending: _ } => self.apply_effects(),
            GameState::EndTurn => self.end_turn(),
            _ => Ok(()),
        }
    }

    pub fn play_card(&mut self, action: Action) -> TurnResult {
        self.player_turn(action)?;

        // self.state = GameState::ApplyEffects;
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

        // self.state = GameState::ApplyEffects;
        Ok(())
    }

    fn apply_effects(&mut self) -> TurnResult {
        let GameState::ApplyEffects { ref mut pending } = self.state else {
            return Err("invalid state".to_string());
        };

        let Some(effect_instance) = pending.pop_front() else {
            self.state = GameState::EndTurn;
            return Ok(());
        };

        match effect_instance.effect {
            super::effect::Effect::Attack => self.attack(effect_instance),
            super::effect::Effect::Capture => self.capture(effect_instance),
        }
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
        CARDS.sample(rng, MAX_HAND_CARDS).map(Card::new).collect()
    }

    fn player_turn(&mut self, action: Action) -> TurnResult {
        if !self.board.is_available(action.target) {
            return Err("position is not available".to_string());
        }

        let card = self
            .player
            .pop_card(action.card_id)
            .ok_or("card id not found")?;

        self.place_card(card, self.player.id, action.target)
    }

    fn cpu_turn(&mut self) -> TurnResult {
        let target = self
            .board
            .find_available(self.rng)
            .ok_or("unable to find a position")?;

        let card = self
            .cpu
            .hand
            .take_random(self.rng)
            .ok_or("unable to draw a card from cpu")?;

        self.place_card(card, self.cpu.id, target)
    }

    fn place_card(&mut self, card: Card, owner_id: u64, target: Position) -> TurnResult {
        self.board.place_card(card, target, owner_id)?;
        let effects = generate_effects(GenerateEffectsParams {
            position: target,
            board: &self.board,
        })?;

        self.state = GameState::ApplyEffects { pending: effects };

        Ok(())
    }

    fn start_game(&mut self) -> TurnResult {
        self.active_player = match self.rng.random_bool(0.5) {
            true => ActivePlayer::Player,
            false => ActivePlayer::Cpu,
        };

        self.state = GameState::StartTurn;

        Ok(())
    }

    fn get_active_player(&self) -> Result<&Player, String> {
        match self.active_player {
            ActivePlayer::Cpu => Ok(&self.cpu),
            ActivePlayer::Player => Ok(&self.player),
            _ => Err("Invalid active player".to_string()),
        }
    }

    fn attack(&mut self, effect_instance: EffectInstance) -> TurnResult {
        let (source_tc, source_pos) = self
            .board
            .find_placed_by_id(effect_instance.source_card_id)
            .ok_or("Cannot find source")?;

        let (target_tc, target_pos) = self
            .board
            .find_placed_by_id(effect_instance.target_card_id)
            .ok_or("Cannot find target")?;

        let result = attack(AttackParams {
            attacker: &source_tc.card,
            defender: &target_tc.card,
        })?;

        match result {
            AttackOutcome::Win => self.board.set_owner(target_pos, source_tc.owner_id),
            AttackOutcome::Lose => {
                self.state = GameState::EndTurn;
                self.board.set_owner(source_pos, target_tc.owner_id)
            }
        }?;

        Ok(())
    }

    fn capture(&mut self, effect_instance: EffectInstance) -> TurnResult {
        let active_player = self.get_active_player()?;
        let (_, target) = self
            .board
            .find_placed_by_id(effect_instance.target_card_id)
            .ok_or("Unable to find target")?;

        self.board.set_owner(target, active_player.id)?;

        Ok(())
    }
}
