use std::collections::{HashSet, VecDeque};

use crate::{
    ai::{CpuMoveInput, choose_random_action},
    assets::CARDS,
    models::{
        core::{Player, board::Board, card::Card, geometry::Position},
        session::{
            ActivePlayer, BoardTileSnapshot, CombatResult, GameAction, GameError, GameEvent,
            GamePhase, GameResult, GameSnapshot, GameUpdate, InteractionState,
            OwnershipChangeReason, PendingEffect, PlayerSide, SessionPhase,
        },
    },
    rules::{
        capture::{Capture, CaptureKind, apply_capture},
        combat::{CombatOutcome, CombatParams, resolve_combat},
        combo::{ComboCapture, discover_combo_captures},
        placement::{
            PlacementInteractionKind, discover_interactions, is_legal_position, legal_positions,
        },
    },
    utils::{constants::MAX_HAND_CARDS, random::GameRng},
};

use super::Effect;

type TurnResult = Result<(), String>;

fn enqueue_combo_captures(pending: &mut VecDeque<PendingEffect>, combo_captures: &[ComboCapture]) {
    for capture in combo_captures.iter().rev() {
        pending.retain(|effect| effect.target_card_id != capture.target_card_id);
        pending.push_front(PendingEffect::new(
            capture.source_card_id,
            capture.target_card_id,
            Effect::ComboCapture,
        ));
    }
}

struct ResolvedCapture {
    card_id: u64,
    new_owner_id: u64,
    reason: OwnershipChangeReason,
    combo_origin: Option<Position>,
    ends_turn: bool,
}

pub struct GameSession {
    pub board: Board,
    pub player: Player,
    pub cpu: Player,
    pub state: GamePhase,
    pub active_player: ActivePlayer,
    rng: GameRng,
    turn_announced: bool,
    turn_ended_announced: bool,
    resolved_capture: Option<ResolvedCapture>,
}

pub type Game = GameSession;

impl GameSession {
    pub fn new(block_density: f64, mut rng: GameRng) -> Self {
        let board = Board::build(block_density, &mut rng);

        let mut used_card_ids = HashSet::new();
        let player = Self::build_player(false, &mut rng, &mut used_card_ids);
        let cpu = Self::build_player(true, &mut rng, &mut used_card_ids);

        Self {
            board,
            player,
            cpu,
            rng,
            state: GamePhase::NotStarted,
            active_player: ActivePlayer::None,
            turn_announced: false,
            turn_ended_announced: false,
            resolved_capture: None,
        }
    }

    pub fn snapshot(&self) -> GameSnapshot {
        let board = (0..crate::utils::constants::BOARD_SIZE)
            .flat_map(|row| self.board.row(row))
            .map(|tile| match tile {
                crate::models::core::board::Tile::Empty => BoardTileSnapshot::Empty,
                crate::models::core::board::Tile::Blocked => BoardTileSnapshot::Blocked,
                crate::models::core::board::Tile::Occupied(board_card) => {
                    BoardTileSnapshot::Occupied {
                        owner: self.side_for_owner(board_card.owner_id),
                        card: (&board_card.card).into(),
                    }
                }
            })
            .collect();
        let legal_actions = if self.interaction_state() == InteractionState::AwaitingPlayerAction {
            legal_positions(&self.board)
                .into_iter()
                .flat_map(|position| {
                    self.player
                        .hand
                        .iter()
                        .map(move |card| GameAction::PlayCard {
                            card_id: card.id,
                            position,
                        })
                })
                .collect()
        } else {
            Vec::new()
        };

        GameSnapshot {
            board,
            human_hand: self.player.hand.iter().map(Into::into).collect(),
            cpu_hand_count: self.cpu.hand.len(),
            human_score: self.player_score(),
            cpu_score: self.cpu_score(),
            active_player: self.active_side(),
            phase: match self.state {
                GamePhase::NotStarted => SessionPhase::NotStarted,
                GamePhase::StartTurn | GamePhase::EndTurn => SessionPhase::Turn,
                GamePhase::ApplyEffects { .. } => SessionPhase::Resolving,
                GamePhase::Finished => SessionPhase::Finished,
            },
            result: (self.state == GamePhase::Finished).then(|| self.result()),
            legal_actions,
        }
    }

    pub fn dispatch(&mut self, action: GameAction) -> Result<GameUpdate, GameError> {
        let actual = self.interaction_state();
        if actual != InteractionState::AwaitingPlayerAction {
            return Err(GameError::InvalidInteraction {
                expected: InteractionState::AwaitingPlayerAction,
                actual,
            });
        }
        let card_id = action.card_id();
        let position = action.position();
        let card_index = self.validate_action(action, &self.player.hand)?;
        let card = self.player.hand.remove(card_index);
        self.place_card(card, self.player.id, position)?;
        Ok(self.update(GameEvent::CardPlaced {
            player: PlayerSide::Human,
            card_id,
            position,
        }))
    }

    pub fn advance(&mut self) -> Result<GameUpdate, GameError> {
        if self.interaction_state() != InteractionState::Advancing {
            return Err(GameError::InvalidInteraction {
                expected: InteractionState::Advancing,
                actual: self.interaction_state(),
            });
        }
        let event = self.advance_event()?;
        Ok(self.update(event))
    }

    pub fn interaction_state(&self) -> InteractionState {
        if self.state == GamePhase::Finished {
            InteractionState::Finished
        } else if self.state == GamePhase::StartTurn
            && self.active_player == ActivePlayer::Player
            && self.turn_announced
        {
            InteractionState::AwaitingPlayerAction
        } else {
            InteractionState::Advancing
        }
    }

    fn update(&self, event: GameEvent) -> GameUpdate {
        GameUpdate {
            events: vec![event],
            snapshot: self.snapshot(),
            interaction: self.interaction_state(),
        }
    }

    fn advance_event(&mut self) -> Result<GameEvent, GameError> {
        if let Some(capture) = self.resolved_capture.take() {
            return self.apply_resolved_capture(capture);
        }

        match self.state {
            GamePhase::NotStarted => {
                self.start_game()?;
                Ok(GameEvent::GameStarted {
                    first_player: self.active_side().ok_or_else(|| {
                        GameError::Internal("game started without an active player".into())
                    })?,
                })
            }
            GamePhase::StartTurn if !self.turn_announced => {
                self.turn_announced = true;
                Ok(GameEvent::TurnStarted {
                    player: self.active_side().ok_or_else(|| {
                        GameError::Internal("turn started without an active player".into())
                    })?,
                })
            }
            GamePhase::StartTurn => self.advance_cpu_placement(),
            GamePhase::ApplyEffects { .. } => self.advance_effect(),
            GamePhase::EndTurn if !self.turn_ended_announced => {
                self.turn_ended_announced = true;
                Ok(GameEvent::TurnEnded {
                    player: self.active_side().ok_or_else(|| {
                        GameError::Internal("turn ended without an active player".into())
                    })?,
                })
            }
            GamePhase::EndTurn => {
                if self.board.count_empty() == 0
                    || (self.player.hand.is_empty() && self.cpu.hand.is_empty())
                {
                    self.state = GamePhase::Finished;
                    Ok(GameEvent::GameFinished {
                        result: self.result(),
                    })
                } else {
                    self.active_player = match self.active_player {
                        ActivePlayer::Cpu => ActivePlayer::Player,
                        ActivePlayer::Player => ActivePlayer::Cpu,
                        ActivePlayer::None => {
                            return Err(GameError::Internal("invalid active player".into()));
                        }
                    };
                    self.state = GamePhase::StartTurn;
                    self.turn_announced = true;
                    self.turn_ended_announced = false;
                    Ok(GameEvent::TurnStarted {
                        player: self.active_side().expect("active player was just assigned"),
                    })
                }
            }
            GamePhase::Finished => Err(GameError::Internal("finished game cannot advance".into())),
        }
    }

    fn advance_cpu_placement(&mut self) -> Result<GameEvent, GameError> {
        if self.active_player != ActivePlayer::Cpu {
            return Err(GameError::Internal(
                "only the CPU may advance a started turn".into(),
            ));
        }
        let legal_actions = legal_positions(&self.board)
            .into_iter()
            .flat_map(|position| {
                self.cpu.hand.iter().map(move |card| GameAction::PlayCard {
                    card_id: card.id,
                    position,
                })
            })
            .collect::<Vec<_>>();
        let action = choose_random_action(
            CpuMoveInput {
                board: &self.board,
                hand: &self.cpu.hand,
                legal_actions: &legal_actions,
            },
            &mut self.rng,
        )
        .ok_or_else(|| GameError::Internal("unable to choose a cpu action".into()))?;
        let card_id = action.card_id();
        let position = action.position();
        let card_index = self.validate_action(action, &self.cpu.hand)?;
        let card = self.cpu.hand.swap_remove(card_index);
        self.place_card(card, self.cpu.id, position)?;
        Ok(GameEvent::CardPlaced {
            player: PlayerSide::Cpu,
            card_id,
            position,
        })
    }

    fn active_side(&self) -> Option<PlayerSide> {
        match self.active_player {
            ActivePlayer::Player => Some(PlayerSide::Human),
            ActivePlayer::Cpu => Some(PlayerSide::Cpu),
            ActivePlayer::None => None,
        }
    }

    fn side_for_owner(&self, owner_id: u64) -> PlayerSide {
        if owner_id == self.player.id {
            PlayerSide::Human
        } else {
            PlayerSide::Cpu
        }
    }

    fn result(&self) -> GameResult {
        match self.player_score().cmp(&self.cpu_score()) {
            std::cmp::Ordering::Greater => GameResult::Winner(PlayerSide::Human),
            std::cmp::Ordering::Less => GameResult::Winner(PlayerSide::Cpu),
            std::cmp::Ordering::Equal => GameResult::Draw,
        }
    }

    fn advance_effect(&mut self) -> Result<GameEvent, GameError> {
        let pending_effect = {
            let GamePhase::ApplyEffects { pending } = &mut self.state else {
                return Err(GameError::Internal(
                    "effects requested in invalid phase".into(),
                ));
            };
            pending.pop_front()
        };
        let Some(pending_effect) = pending_effect else {
            self.state = GamePhase::EndTurn;
            self.turn_ended_announced = true;
            return Ok(GameEvent::TurnEnded {
                player: self.active_side().ok_or_else(|| {
                    GameError::Internal("effects ended without an active player".into())
                })?,
            });
        };

        match pending_effect.effect {
            Effect::DirectCapture | Effect::ComboCapture => {
                let new_owner_id = self.get_active_player()?.id;
                self.apply_resolved_capture(ResolvedCapture {
                    card_id: pending_effect.target_card_id,
                    new_owner_id,
                    reason: if pending_effect.effect == Effect::ComboCapture {
                        OwnershipChangeReason::Combo
                    } else {
                        OwnershipChangeReason::DirectCapture
                    },
                    combo_origin: None,
                    ends_turn: false,
                })
            }
            Effect::Attack => {
                let (source, source_position) = self
                    .board
                    .position_of_card(pending_effect.source_card_id)
                    .ok_or_else(|| GameError::Internal("cannot find combat source".into()))?;
                let (target, target_position) = self
                    .board
                    .position_of_card(pending_effect.target_card_id)
                    .ok_or_else(|| GameError::Internal("cannot find combat target".into()))?;
                let source_owner = source.owner_id;
                let target_owner = target.owner_id;
                let outcome = resolve_combat(
                    CombatParams {
                        attacker: &source.card,
                        defender: &target.card,
                    },
                    &mut self.rng,
                );
                self.resolved_capture = Some(match outcome {
                    CombatOutcome::Victory => ResolvedCapture {
                        card_id: pending_effect.target_card_id,
                        new_owner_id: source_owner,
                        reason: OwnershipChangeReason::CombatVictory,
                        combo_origin: Some(target_position),
                        ends_turn: false,
                    },
                    CombatOutcome::Defeat => ResolvedCapture {
                        card_id: pending_effect.source_card_id,
                        new_owner_id: target_owner,
                        reason: OwnershipChangeReason::CombatDefeat,
                        combo_origin: None,
                        ends_turn: true,
                    },
                });
                let _ = source_position;
                Ok(GameEvent::CombatResolved {
                    attacker_id: pending_effect.source_card_id,
                    defender_id: pending_effect.target_card_id,
                    outcome: match outcome {
                        CombatOutcome::Victory => CombatResult::AttackerWon,
                        CombatOutcome::Defeat => CombatResult::DefenderWon,
                    },
                })
            }
        }
    }

    fn apply_resolved_capture(&mut self, capture: ResolvedCapture) -> Result<GameEvent, GameError> {
        let outcome = apply_capture(
            &mut self.board,
            Capture {
                card_id: capture.card_id,
                new_owner_id: capture.new_owner_id,
                kind: match capture.reason {
                    OwnershipChangeReason::DirectCapture | OwnershipChangeReason::Combo => {
                        CaptureKind::Direct
                    }
                    OwnershipChangeReason::CombatVictory => CaptureKind::CombatVictory,
                    OwnershipChangeReason::CombatDefeat => CaptureKind::CombatDefeat,
                },
            },
        )
        .map_err(|error| GameError::Internal(error.to_string()))?;

        if let Some(position) = capture.combo_origin {
            let combo_captures = discover_combo_captures(&self.board, position)
                .map_err(|error| GameError::Internal(error.to_string()))?;
            let GamePhase::ApplyEffects { pending } = &mut self.state else {
                return Err(GameError::Internal(
                    "combo discovered outside effects".into(),
                ));
            };
            enqueue_combo_captures(pending, &combo_captures);
        }
        if capture.ends_turn {
            self.state = GamePhase::EndTurn;
            self.turn_ended_announced = false;
        }

        Ok(GameEvent::OwnershipChanged {
            card_id: capture.card_id,
            previous_owner: self.side_for_owner(outcome.previous_owner_id),
            new_owner: self.side_for_owner(capture.new_owner_id),
            reason: capture.reason,
        })
    }

    pub fn player_hand(&self) -> &[Card] {
        &self.player.hand
    }

    pub fn run(&mut self) -> TurnResult {
        match self.state {
            GamePhase::NotStarted => Ok(self.start_game()?),
            GamePhase::StartTurn => self.start_turn(),
            GamePhase::ApplyEffects { pending: _ } => self.apply_effects(),
            GamePhase::EndTurn => self.end_turn(),
            _ => Ok(()),
        }
    }

    pub fn play_card(&mut self, action: GameAction) -> TurnResult {
        self.player_turn(action)?;
        Ok(())
    }

    pub fn awaiting_input(&self) -> bool {
        self.state == GamePhase::StartTurn && self.active_player == ActivePlayer::Player
    }

    pub fn player_score(&self) -> usize {
        self.board.score(self.player.id)
    }

    pub fn cpu_score(&self) -> usize {
        self.board.score(self.cpu.id)
    }

    fn start_turn(&mut self) -> TurnResult {
        if self.active_player != ActivePlayer::Cpu {
            return Err("Active player should be CPU".to_string());
        }

        self.cpu_turn()?;

        // self.state = GamePhase::ApplyEffects;
        Ok(())
    }

    fn apply_effects(&mut self) -> TurnResult {
        let GamePhase::ApplyEffects { ref mut pending } = self.state else {
            return Err("invalid state".to_string());
        };

        let Some(pending_effect) = pending.pop_front() else {
            self.state = GamePhase::EndTurn;
            return Ok(());
        };

        match pending_effect.effect {
            Effect::Attack => self.attack(pending_effect),
            Effect::DirectCapture | Effect::ComboCapture => self.capture(pending_effect),
        }
    }

    fn end_turn(&mut self) -> TurnResult {
        if self.board.count_empty() == 0 {
            self.state = GamePhase::Finished;
            return Ok(());
        }

        if self.player.hand.is_empty() && self.cpu.hand.is_empty() {
            self.state = GamePhase::Finished;
            return Ok(());
        }

        self.active_player = match self.active_player {
            ActivePlayer::Cpu => ActivePlayer::Player,
            ActivePlayer::Player => ActivePlayer::Cpu,
            _ => return Err("Invalid active player".to_string()),
        };

        self.state = GamePhase::StartTurn;

        Ok(())
    }

    fn build_player(is_cpu: bool, rng: &mut GameRng, used_card_ids: &mut HashSet<u64>) -> Player {
        let id: u64 = rng.next_u64();
        let hand = Self::build_hand(rng, used_card_ids);
        let name = match is_cpu {
            true => "CPU".to_string(),
            false => "Player".to_string(),
        };

        Player { id, name, hand }
    }

    fn build_hand(rng: &mut GameRng, used_card_ids: &mut HashSet<u64>) -> Vec<Card> {
        rng.sample(&CARDS, MAX_HAND_CARDS)
            .into_iter()
            .map(|definition| {
                loop {
                    let card = Card::new(definition, rng);
                    if used_card_ids.insert(card.id) {
                        break card;
                    }
                }
            })
            .collect()
    }

    fn player_turn(&mut self, action: GameAction) -> TurnResult {
        let card_index = self.validate_action(action, &self.player.hand)?;
        let card = self.player.hand.remove(card_index);

        Ok(self.place_card(card, self.player.id, action.position())?)
    }

    fn cpu_turn(&mut self) -> TurnResult {
        let legal_actions = legal_positions(&self.board)
            .into_iter()
            .flat_map(|target| {
                self.cpu
                    .hand
                    .iter()
                    .map(move |card| GameAction::new(card.id, target))
            })
            .collect::<Vec<_>>();
        let action = choose_random_action(
            CpuMoveInput {
                board: &self.board,
                hand: &self.cpu.hand,
                legal_actions: &legal_actions,
            },
            &mut self.rng,
        )
        .ok_or("unable to choose a cpu action")?;
        let card_index = self.validate_action(action, &self.cpu.hand)?;
        let card = self.cpu.hand.swap_remove(card_index);

        Ok(self.place_card(card, self.cpu.id, action.position())?)
    }

    fn validate_action(&self, action: GameAction, hand: &[Card]) -> Result<usize, GameError> {
        if !is_legal_position(&self.board, action.position()) {
            return Err(GameError::IllegalPosition(action.position()));
        }

        hand.iter()
            .position(|card| card.id == action.card_id())
            .ok_or(GameError::CardNotInHand(action.card_id()))
    }

    fn place_card(&mut self, card: Card, owner_id: u64, target: Position) -> Result<(), GameError> {
        self.board
            .place_card(card, target, owner_id)
            .map_err(|error| GameError::Internal(error.to_string()))?;
        let effects = discover_interactions(&self.board, target)
            .map_err(|error| GameError::Internal(error.to_string()))?
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
            .collect();

        self.state = GamePhase::ApplyEffects { pending: effects };

        Ok(())
    }

    fn start_game(&mut self) -> Result<(), GameError> {
        self.active_player = match self.rng.random_bool(0.5) {
            true => ActivePlayer::Player,
            false => ActivePlayer::Cpu,
        };

        self.state = GamePhase::StartTurn;
        self.turn_announced = false;

        Ok(())
    }

    fn get_active_player(&self) -> Result<&Player, String> {
        match self.active_player {
            ActivePlayer::Cpu => Ok(&self.cpu),
            ActivePlayer::Player => Ok(&self.player),
            _ => Err("Invalid active player".to_string()),
        }
    }

    fn attack(&mut self, pending_effect: PendingEffect) -> TurnResult {
        let (source_tc, source_pos) = self
            .board
            .position_of_card(pending_effect.source_card_id)
            .ok_or("Cannot find source")?;

        let (target_tc, target_pos) = self
            .board
            .position_of_card(pending_effect.target_card_id)
            .ok_or("Cannot find target")?;

        let result = resolve_combat(
            CombatParams {
                attacker: &source_tc.card,
                defender: &target_tc.card,
            },
            &mut self.rng,
        );

        match result {
            CombatOutcome::Victory => self.on_victory(source_tc.owner_id, target_pos),
            CombatOutcome::Defeat => self.on_defeat(target_tc.owner_id, source_pos),
        }
    }

    fn capture(&mut self, pending_effect: PendingEffect) -> TurnResult {
        let active_player_id = self.get_active_player()?.id;
        apply_capture(
            &mut self.board,
            Capture {
                card_id: pending_effect.target_card_id,
                new_owner_id: active_player_id,
                kind: CaptureKind::Direct,
            },
        )
        .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn on_victory(&mut self, owner_id: u64, pos: Position) -> TurnResult {
        let target_card_id = self
            .board
            .get_card(pos)
            .ok_or("Unable to find target")?
            .card
            .id;
        apply_capture(
            &mut self.board,
            Capture {
                card_id: target_card_id,
                new_owner_id: owner_id,
                kind: CaptureKind::CombatVictory,
            },
        )
        .map_err(|error| error.to_string())?;

        let combo_captures =
            discover_combo_captures(&self.board, pos).map_err(|error| error.to_string())?;

        let GamePhase::ApplyEffects { pending } = &mut self.state else {
            return Err("skip side effects".into());
        };

        enqueue_combo_captures(pending, &combo_captures);

        Ok(())
    }

    fn on_defeat(&mut self, owner_id: u64, pos: Position) -> TurnResult {
        let source_card_id = self
            .board
            .get_card(pos)
            .ok_or("Unable to find source")?
            .card
            .id;
        apply_capture(
            &mut self.board,
            Capture {
                card_id: source_card_id,
                new_owner_id: owner_id,
                kind: CaptureKind::CombatDefeat,
            },
        )
        .map_err(|error| error.to_string())?;
        self.state = GamePhase::EndTurn;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{
            core::{
                board::{BoardCard, Tile},
                card::{BattleClass, Card},
            },
            session::Effect,
        },
        test_support::{card, empty_tiles},
        utils::constants::MAX_HAND_CARDS,
    };

    fn card_signature(card: &Card) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}:{}",
            card.id,
            card.arrows,
            card.asset.name,
            card.stats.attack,
            card.stats.battle_class,
            card.stats.phys_defense,
            card.stats.mag_defense
        )
    }

    fn game_signature(game: &Game) -> (Vec<String>, Vec<String>, Vec<String>, String) {
        let board = (0..crate::utils::constants::BOARD_SIZE)
            .flat_map(|row| game.board.row(row))
            .map(|tile| match tile {
                Tile::Empty => "empty".to_string(),
                Tile::Blocked => "blocked".to_string(),
                Tile::Occupied(board_card) => {
                    format!(
                        "{}:{}",
                        board_card.owner_id,
                        card_signature(&board_card.card)
                    )
                }
            })
            .collect();
        let player_hand = game.player.hand.iter().map(card_signature).collect();
        let cpu_hand = game.cpu.hand.iter().map(card_signature).collect();
        let active_player = match game.active_player {
            ActivePlayer::None => "none",
            ActivePlayer::Player => "player",
            ActivePlayer::Cpu => "cpu",
        };
        let session = format!(
            "{}:{}:{:?}:{active_player}",
            game.player.id, game.cpu.id, game.state
        );

        (board, player_hand, cpu_hand, session)
    }

    fn configured_game(seed: u64) -> Game {
        let mut game = Game::new(0.0, GameRng::from_seed(seed));
        game.board = Board::from_tiles(empty_tiles());
        game.player.id = 1;
        game.player.hand = (0..MAX_HAND_CARDS)
            .map(|i| card(10 + i as u64, 0, BattleClass::Physical, 1, 1, 1))
            .collect();
        game.cpu.id = 2;
        game.cpu.hand = (0..MAX_HAND_CARDS)
            .map(|i| card(20 + i as u64, 0, BattleClass::Physical, 1, 1, 1))
            .collect();
        game
    }

    #[test]
    fn same_seed_and_action_sequence_reproduce_the_entire_game() {
        const SEED: u64 = 0x5EED_CAFE;
        let mut first = Game::new(0.25, GameRng::from_seed(SEED));
        let mut second = Game::new(0.25, GameRng::from_seed(SEED));

        for step in 0..100 {
            assert_eq!(
                game_signature(&first),
                game_signature(&second),
                "seed {SEED} diverged before step {step}"
            );

            if first.state == GamePhase::Finished {
                return;
            }

            if first.awaiting_input() {
                let card_id = first.player.hand[0].id;
                let target = legal_positions(&first.board)[0];
                let action = GameAction::new(card_id, target);
                first.play_card(action).unwrap();
                second.play_card(GameAction::new(card_id, target)).unwrap();
            } else {
                first.run().unwrap();
                second.run().unwrap();
            }
        }

        panic!("seed {SEED} did not finish within 100 steps");
    }

    #[test]
    fn game_start_selects_an_active_player_and_enters_start_turn() {
        let mut game = configured_game(1);
        assert_eq!(game.state, GamePhase::NotStarted);
        game.run().unwrap();
        assert_eq!(game.state, GamePhase::StartTurn);
        assert!(matches!(
            game.active_player,
            ActivePlayer::Player | ActivePlayer::Cpu
        ));
        assert_eq!(
            game.awaiting_input(),
            game.active_player == ActivePlayer::Player
        );
    }

    #[test]
    fn player_turn_rejects_unavailable_position_without_removing_card() {
        let mut game = configured_game(2);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Blocked;
        game.board = Board::from_tiles(tiles);
        let card_id = game.player.hand[0].id;
        let before = game.player.hand.len();
        assert!(
            game.play_card(GameAction::new(card_id, Position::new(0, 0)))
                .is_err()
        );
        assert_eq!(game.player.hand.len(), before);
        assert_eq!(game.state, GamePhase::StartTurn);
    }

    #[test]
    fn player_placement_enters_effect_processing_then_end_turn() {
        let mut game = configured_game(3);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        let card_id = game.player.hand[0].id;
        game.play_card(GameAction::new(card_id, Position::new(0, 0)))
            .unwrap();
        assert_eq!(game.player.hand.len(), MAX_HAND_CARDS - 1);
        assert!(
            matches!(game.state, GamePhase::ApplyEffects { ref pending } if pending.is_empty())
        );
        assert_eq!(game.player_score(), 1);
        game.run().unwrap();
        assert_eq!(game.state, GamePhase::EndTurn);
        game.run().unwrap();
        assert_eq!(game.state, GamePhase::StartTurn);
        assert!(game.active_player == ActivePlayer::Cpu);
    }

    #[test]
    fn cpu_turn_chooses_an_available_position_and_consumes_one_card() {
        let mut game = configured_game(4);
        game.active_player = ActivePlayer::Cpu;
        game.state = GamePhase::StartTurn;
        game.run().unwrap();
        assert_eq!(game.cpu.hand.len(), MAX_HAND_CARDS - 1);
        assert_eq!(game.cpu_score(), 1);
        assert!(matches!(game.state, GamePhase::ApplyEffects { .. }));
    }

    #[test]
    fn capture_effect_changes_target_owner_one_effect_per_run() {
        let mut game = configured_game(5);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Occupied(BoardCard {
            owner_id: 1,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            owner_id: 2,
            card: card(20, 0, BattleClass::Physical, 0, 0, 0),
        });
        game.board = Board::from_tiles(tiles);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::ApplyEffects {
            pending: VecDeque::from([PendingEffect::new(10, 20, Effect::DirectCapture)]),
        };
        game.run().unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 1)).unwrap().owner_id,
            1
        );
        assert!(
            matches!(game.state, GamePhase::ApplyEffects { ref pending } if pending.is_empty())
        );
        game.run().unwrap();
        assert_eq!(game.state, GamePhase::EndTurn);
    }

    #[test]
    fn combo_captures_precede_and_replace_pending_effects_for_the_same_targets() {
        let mut pending = VecDeque::from([
            PendingEffect::new(99, 20, Effect::Attack),
            PendingEffect::new(99, 30, Effect::DirectCapture),
        ]);

        enqueue_combo_captures(
            &mut pending,
            &[
                ComboCapture {
                    source_card_id: 10,
                    target_card_id: 40,
                },
                ComboCapture {
                    source_card_id: 10,
                    target_card_id: 20,
                },
            ],
        );

        assert_eq!(pending.len(), 3);
        assert_eq!(
            pending
                .iter()
                .map(|effect| (effect.source_card_id, effect.target_card_id))
                .collect::<Vec<_>>(),
            vec![(10, 40), (10, 20), (99, 30)]
        );
        assert!(
            pending
                .iter()
                .take(2)
                .all(|effect| effect.effect == Effect::ComboCapture)
        );
    }

    #[test]
    fn victory_propagates_capture_while_defeat_flips_source_and_ends_turn() {
        let mut game = configured_game(6);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Occupied(BoardCard {
            owner_id: 1,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            owner_id: 2,
            card: card(
                20,
                1 << crate::models::core::geometry::Direction::E as u8,
                BattleClass::Physical,
                0,
                0,
                0,
            ),
        });
        tiles[2] = Tile::Occupied(BoardCard {
            owner_id: 2,
            card: card(30, 0, BattleClass::Physical, 0, 0, 0),
        });
        game.board = Board::from_tiles(tiles);
        game.state = GamePhase::ApplyEffects {
            pending: VecDeque::new(),
        };
        game.on_victory(1, Position::new(0, 1)).unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 1)).unwrap().owner_id,
            1
        );
        assert!(
            matches!(game.state, GamePhase::ApplyEffects { ref pending } if pending.front().is_some_and(|effect| effect.target_card_id == 30))
        );

        game.on_defeat(2, Position::new(0, 0)).unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 0)).unwrap().owner_id,
            2
        );
        assert_eq!(game.state, GamePhase::EndTurn);
    }

    #[test]
    fn end_turn_finishes_for_full_board_or_two_empty_hands() {
        let mut game = configured_game(7);
        game.state = GamePhase::EndTurn;
        game.player.hand.clear();
        game.cpu.hand.clear();
        game.run().unwrap();
        assert_eq!(game.state, GamePhase::Finished);

        let tiles = std::array::from_fn(|index| {
            Tile::Occupied(BoardCard {
                owner_id: 1,
                card: card(100 + index as u64, 0, BattleClass::Physical, 0, 0, 0),
            })
        });
        game.board = Board::from_tiles(tiles);
        game.player
            .hand
            .push(card(1, 0, BattleClass::Physical, 0, 0, 0));
        game.state = GamePhase::EndTurn;
        game.run().unwrap();
        assert_eq!(game.state, GamePhase::Finished);
    }

    #[test]
    fn automated_full_game_reaches_completion_and_preserves_card_totals() {
        let mut game = configured_game(8);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        let mut steps = 0;
        while game.state != GamePhase::Finished {
            if game.awaiting_input() {
                let card_id = game.player.hand[0].id;
                let target = (0..16)
                    .filter_map(crate::utils::helpers::idx2pos)
                    .find(|position| game.board.is_available(*position))
                    .unwrap();
                game.play_card(GameAction::new(card_id, target)).unwrap();
            } else {
                game.run().unwrap();
            }
            steps += 1;
            assert!(steps < 100);
        }
        assert!(game.player.hand.is_empty());
        assert!(game.cpu.hand.is_empty());
        assert_eq!(game.player_score() + game.cpu_score(), MAX_HAND_CARDS * 2);
        assert_eq!(game.board.count_empty(), 16 - MAX_HAND_CARDS * 2);
    }

    #[test]
    fn invalid_missing_card_does_not_change_board() {
        let mut game = configured_game(9);
        assert!(
            game.play_card(GameAction::new(999, Position::new(0, 0)))
                .is_err()
        );
        assert!(game.board.is_available(Position::new(0, 0)));
        assert_eq!(game.player.hand.len(), MAX_HAND_CARDS);
    }

    #[test]
    fn public_contract_drives_a_complete_seeded_game_without_hidden_cpu_cards() {
        let mut game = GameSession::new(0.25, GameRng::from_seed(0xC0DE_CAFE));
        let initial = game.snapshot();
        assert_eq!(initial.cpu_hand_count, MAX_HAND_CARDS);
        assert_eq!(initial.human_hand.len(), MAX_HAND_CARDS);

        let mut updates = 0;
        while game.interaction_state() != InteractionState::Finished {
            let update = match game.interaction_state() {
                InteractionState::AwaitingPlayerAction => {
                    let action = game.snapshot().legal_actions[0];
                    game.dispatch(action).unwrap()
                }
                InteractionState::Advancing => game.advance().unwrap(),
                InteractionState::Finished => unreachable!(),
            };
            assert_eq!(update.events.len(), 1);
            assert_eq!(update.snapshot, game.snapshot());
            assert_eq!(update.interaction, game.interaction_state());
            updates += 1;
            assert!(updates < 100);
        }

        let final_snapshot = game.snapshot();
        assert!(final_snapshot.human_hand.is_empty());
        assert_eq!(final_snapshot.cpu_hand_count, 0);
        assert!(final_snapshot.result.is_some());
    }

    #[test]
    fn invalid_dispatch_is_atomic_and_returns_a_typed_error() {
        let mut game = configured_game(10);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        game.turn_announced = true;
        let before = game.snapshot();

        let error = game
            .dispatch(GameAction::PlayCard {
                card_id: 999,
                position: Position::new(0, 0),
            })
            .unwrap_err();

        assert_eq!(error, GameError::CardNotInHand(999));
        assert_eq!(game.snapshot(), before);
    }

    #[test]
    fn combat_and_ownership_are_separate_state_after_event_transitions() {
        let mut game = configured_game(11);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Occupied(BoardCard {
            owner_id: 1,
            card: card(10, 0, BattleClass::Physical, 100, 100, 100),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            owner_id: 2,
            card: card(20, 0, BattleClass::Physical, 100, 100, 100),
        });
        game.board = Board::from_tiles(tiles);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::ApplyEffects {
            pending: VecDeque::from([PendingEffect::new(10, 20, Effect::Attack)]),
        };

        let combat = game.advance().unwrap();
        let (captured_id, owner_before) = match combat.events[0] {
            GameEvent::CombatResolved {
                attacker_id: 10,
                defender_id: 20,
                outcome: CombatResult::AttackerWon,
            } => (20, 2),
            GameEvent::CombatResolved {
                attacker_id: 10,
                defender_id: 20,
                outcome: CombatResult::DefenderWon,
            } => (10, 1),
            ref event => panic!("unexpected event: {event:?}"),
        };
        assert_eq!(
            game.board.position_of_card(captured_id).unwrap().0.owner_id,
            owner_before
        );

        let ownership = game.advance().unwrap();
        assert!(matches!(
            ownership.events[0],
            GameEvent::OwnershipChanged { card_id, .. } if card_id == captured_id
        ));
        assert_ne!(
            game.board.position_of_card(captured_id).unwrap().0.owner_id,
            owner_before
        );
    }

    #[test]
    fn generated_runtime_card_ids_are_unique_within_a_session() {
        for seed in 0..64 {
            let game = GameSession::new(0.0, GameRng::from_seed(seed));
            let ids = game
                .player
                .hand
                .iter()
                .chain(&game.cpu.hand)
                .map(|card| card.id)
                .collect::<HashSet<_>>();
            assert_eq!(ids.len(), MAX_HAND_CARDS * 2, "seed {seed}");
        }
    }
}
