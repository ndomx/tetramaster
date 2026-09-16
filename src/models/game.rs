use std::collections::VecDeque;

use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::cards::CARDS,
    min,
    models::{
        action::Action, active_player::ActivePlayer, board::Board, card::Card,
        direction::Direction, effect::Effect, effect_instance::EffectInstance,
        game_state::GameState, placed_card::PlacedCard, player::Player, position::Position,
    },
    utils::{constants::MAX_HAND_CARDS, random::VecRandomExt},
};

use super::battle_class::BattleClass;

type TurnResult = Result<(), String>;

pub struct Game<'a> {
    pub board: Board,
    pub player: Player,
    pub cpu: Player,
    pub state: GameState,
    pub active_player: ActivePlayer,
    placed_cards: Vec<PlacedCard>,
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
            placed_cards: vec![],
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

        let player = &mut self.player;
        let Some(card) = player.pop_card(action.card_id) else {
            return Err("card id not found".to_string());
        };

        self.board.place_card(&card, action.target, player.id)?;

        let effects = self.spawn_effects(&card, action.target)?;
        self.state = GameState::ApplyEffects { pending: effects };

        self.placed_cards.push(PlacedCard::new(card, action.target));

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

        self.board.place_card(&card, target, cpu.id)?;

        let effects = self.spawn_effects(&card, target)?;
        self.state = GameState::ApplyEffects { pending: effects };

        self.placed_cards.push(PlacedCard::new(card, target));
        Ok(())
    }

    fn spawn_effects(
        &self,
        card: &Card,
        target: Position,
    ) -> Result<VecDeque<EffectInstance>, String> {
        let effects = self
            .neighboring_enemies(target, card.facing())?
            .iter()
            .filter_map(|&(card_id, dir)| {
                self.find_placed(card_id).map(|c| {
                    let e = match c.card.is_facing(dir.opposite()) {
                        true => Effect::Attack,
                        false => Effect::Capture,
                    };

                    EffectInstance::new(card.id, card_id, e)
                })
            })
            .collect();

        Ok(effects)
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

    pub fn find_placed(&self, card_id: u64) -> Option<&PlacedCard> {
        self.placed_cards.iter().find(|pc| pc.card.id == card_id)
    }

    fn neighboring_enemies(
        &self,
        pos: Position,
        dirs: Vec<Direction>,
    ) -> Result<Vec<(u64, Direction)>, String> {
        let owner_id = self.get_active_player()?.id;

        let enemies = dirs
            .iter()
            .filter_map(|dir| self.board.get_relative(pos, dir).map(|tc| (tc, dir)))
            .filter(|(tc, _)| tc.owner_id != owner_id)
            .map(|(tc, &dir)| (tc.card_id, dir))
            .collect();

        Ok(enemies)
    }

    fn attack(&mut self, effect_instance: EffectInstance) -> TurnResult {
        let source = self
            .find_placed(effect_instance.source_card_id)
            .ok_or("Cannot find source")?;

        let challenger = &source.card;

        let atk_pwr = challenger.stats.attack + rand::random_range(0..16u8);
        let atk_penalty = rand::random_range(0..=atk_pwr);
        let atk = atk_pwr.saturating_sub(atk_penalty);

        let target = self
            .find_placed(effect_instance.target_card_id)
            .ok_or("Cannot find target")?;

        let defending = &target.card;
        let def_stat = match challenger.stats.battle_class {
            BattleClass::Physical => defending.stats.phys_defense,
            BattleClass::Magic => defending.stats.mag_defense,
            BattleClass::Flexible => {
                min!(defending.stats.phys_defense, defending.stats.mag_defense)
            }
            BattleClass::Assault => min!(
                defending.stats.phys_defense,
                defending.stats.mag_defense,
                defending.stats.attack
            ),
        };

        let def_pwr = def_stat + rand::random_range(0..16u8);
        let def_penalty = rand::random_range(0..=def_pwr);
        let def = def_pwr.saturating_sub(def_penalty);

        let (challenger_id, defendant_id) = match self.active_player {
            ActivePlayer::Player => Ok((self.player.id, self.cpu.id)),
            ActivePlayer::Cpu => Ok((self.cpu.id, self.player.id)),
            ActivePlayer::None => Err("invalid active player".to_string()),
        }?;

        println!("atk={}, def={}", atk, def);

        match atk > def {
            true => self.board.swap_owner(target.pos, challenger_id),
            false => self.board.swap_owner(source.pos, defendant_id),
        }?;

        Ok(())
    }

    fn capture(&mut self, effect_instance: EffectInstance) -> TurnResult {
        let target = self
            .find_placed(effect_instance.target_card_id)
            .ok_or("Cannot find target")?;

        let active_player = self.get_active_player()?;

        self.board.swap_owner(target.pos, active_player.id)?;

        Ok(())
    }
}
