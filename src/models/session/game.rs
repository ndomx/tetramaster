use std::collections::{HashSet, VecDeque};

use crate::{
    ai::{CpuMoveInput, choose_random_action},
    assets::CARDS,
    models::{
        core::{
            Player,
            board::{Board, BoardSide},
            card::Card,
            geometry::Position,
        },
        session::{
            ActivePlayer, BoardTileSnapshot, CombatResult, ControlChangeReason, GameAction,
            GameError, GameEvent, GamePhase, GameResult, GameSnapshot, GameUpdate,
            InteractionState, PendingEffect, PlayerSide, SessionPhase,
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
    new_controller: BoardSide,
    reason: ControlChangeReason,
    combo_origin: Option<Position>,
    ends_turn: bool,
}

pub struct GameSession {
    board: Board,
    human_player: Player,
    cpu_player: Player,
    state: GamePhase,
    active_player: ActivePlayer,
    rng: GameRng,
    turn_announced: bool,
    turn_ended_announced: bool,
    resolved_capture: Option<ResolvedCapture>,
}

impl GameSession {
    pub fn new(block_density: f64, mut rng: GameRng) -> Self {
        let board = Board::build(block_density, &mut rng);

        let mut used_card_ids = HashSet::new();
        let human_player = Self::build_player(false, &mut rng, &mut used_card_ids);
        let cpu_player = Self::build_player(true, &mut rng, &mut used_card_ids);

        Self {
            board,
            human_player,
            cpu_player,
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
                        controller: board_card.controller,
                        card: (&board_card.card).into(),
                    }
                }
            })
            .collect();
        let legal_actions = if self.interaction_state() == InteractionState::AwaitingPlayerAction {
            legal_positions(&self.board)
                .into_iter()
                .flat_map(|position| {
                    self.human_player
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
            human_hand: self.human_player.hand.iter().map(Into::into).collect(),
            cpu_hand_count: self.cpu_player.hand.len(),
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
        let card_index = self.validate_action(action, &self.human_player.hand)?;
        let card = self.human_player.hand.remove(card_index);
        self.place_card(card, self.human_player.board_side, position)?;
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
                    || (self.human_player.hand.is_empty() && self.cpu_player.hand.is_empty())
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
                self.cpu_player
                    .hand
                    .iter()
                    .map(move |card| GameAction::PlayCard {
                        card_id: card.id,
                        position,
                    })
            })
            .collect::<Vec<_>>();
        let action = choose_random_action(
            CpuMoveInput {
                legal_actions: &legal_actions,
            },
            &mut self.rng,
        )
        .ok_or_else(|| GameError::Internal("unable to choose a cpu action".into()))?;
        let card_id = action.card_id();
        let position = action.position();
        let card_index = self.validate_action(action, &self.cpu_player.hand)?;
        let card = self.cpu_player.hand.swap_remove(card_index);
        self.place_card(card, self.cpu_player.board_side, position)?;
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
                let new_controller = self.active_board_side()?;
                self.apply_resolved_capture(ResolvedCapture {
                    card_id: pending_effect.target_card_id,
                    new_controller,
                    reason: if pending_effect.effect == Effect::ComboCapture {
                        ControlChangeReason::Combo
                    } else {
                        ControlChangeReason::DirectCapture
                    },
                    combo_origin: None,
                    ends_turn: false,
                })
            }
            Effect::Attack => {
                let (source, _) = self
                    .board
                    .position_of_card(pending_effect.source_card_id)
                    .ok_or_else(|| GameError::Internal("cannot find combat source".into()))?;
                let (target, target_position) = self
                    .board
                    .position_of_card(pending_effect.target_card_id)
                    .ok_or_else(|| GameError::Internal("cannot find combat target".into()))?;
                let source_controller = source.controller;
                let target_controller = target.controller;
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
                        new_controller: source_controller,
                        reason: ControlChangeReason::CombatVictory,
                        combo_origin: Some(target_position),
                        ends_turn: false,
                    },
                    CombatOutcome::Defeat => ResolvedCapture {
                        card_id: pending_effect.source_card_id,
                        new_controller: target_controller,
                        reason: ControlChangeReason::CombatDefeat,
                        combo_origin: None,
                        ends_turn: true,
                    },
                });
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
                new_controller: capture.new_controller,
                kind: match capture.reason {
                    ControlChangeReason::DirectCapture | ControlChangeReason::Combo => {
                        CaptureKind::Direct
                    }
                    ControlChangeReason::CombatVictory => CaptureKind::CombatVictory,
                    ControlChangeReason::CombatDefeat => CaptureKind::CombatDefeat,
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

        Ok(GameEvent::ControlChanged {
            card_id: capture.card_id,
            previous_controller: outcome.previous_controller,
            new_controller: capture.new_controller,
            reason: capture.reason,
        })
    }

    fn player_score(&self) -> usize {
        self.board.score(self.human_player.board_side)
    }

    fn cpu_score(&self) -> usize {
        self.board.score(self.cpu_player.board_side)
    }

    fn build_player(is_cpu: bool, rng: &mut GameRng, used_card_ids: &mut HashSet<u64>) -> Player {
        // Preserve the established seeded sequence after removing numeric player IDs.
        let _ = rng.next_u64();
        let hand = Self::build_hand(rng, used_card_ids);
        let board_side = match is_cpu {
            true => BoardSide::Red,
            false => BoardSide::Blue,
        };

        Player { board_side, hand }
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

    fn validate_action(&self, action: GameAction, hand: &[Card]) -> Result<usize, GameError> {
        if !is_legal_position(&self.board, action.position()) {
            return Err(GameError::IllegalPosition(action.position()));
        }

        hand.iter()
            .position(|card| card.id == action.card_id())
            .ok_or(GameError::CardNotInHand(action.card_id()))
    }

    fn place_card(
        &mut self,
        card: Card,
        controller: BoardSide,
        target: Position,
    ) -> Result<(), GameError> {
        self.board
            .place_card(card, target, controller)
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

    fn active_board_side(&self) -> Result<BoardSide, String> {
        match self.active_player {
            ActivePlayer::Cpu => Ok(self.cpu_player.board_side),
            ActivePlayer::Player => Ok(self.human_player.board_side),
            _ => Err("Invalid active player".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{
            core::{
                board::{BoardCard, BoardSide, Tile},
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
            card.definition.name,
            card.stats.attack,
            card.stats.battle_class,
            card.stats.phys_defense,
            card.stats.mag_defense
        )
    }

    fn game_signature(game: &GameSession) -> (Vec<String>, Vec<String>, Vec<String>, String) {
        let board = (0..crate::utils::constants::BOARD_SIZE)
            .flat_map(|row| game.board.row(row))
            .map(|tile| match tile {
                Tile::Empty => "empty".to_string(),
                Tile::Blocked => "blocked".to_string(),
                Tile::Occupied(board_card) => {
                    format!(
                        "{:?}:{}",
                        board_card.controller,
                        card_signature(&board_card.card)
                    )
                }
            })
            .collect();
        let player_hand = game.human_player.hand.iter().map(card_signature).collect();
        let cpu_hand = game.cpu_player.hand.iter().map(card_signature).collect();
        let active_player = match game.active_player {
            ActivePlayer::None => "none",
            ActivePlayer::Player => "player",
            ActivePlayer::Cpu => "cpu",
        };
        let session = format!("{:?}:{active_player}", game.state);

        (board, player_hand, cpu_hand, session)
    }

    fn configured_game(seed: u64) -> GameSession {
        let mut game = GameSession::new(0.0, GameRng::from_seed(seed));
        game.board = Board::from_tiles(empty_tiles());
        game.human_player.hand = (0..MAX_HAND_CARDS)
            .map(|index| card(10 + index as u64, 0, BattleClass::Physical, 1, 1, 1))
            .collect();
        game.cpu_player.hand = (0..MAX_HAND_CARDS)
            .map(|index| card(20 + index as u64, 0, BattleClass::Physical, 1, 1, 1))
            .collect();
        game
    }

    #[test]
    fn same_seed_and_action_sequence_reproduce_the_entire_game() {
        const SEED: u64 = 0x5EED_CAFE;
        let mut first = GameSession::new(0.25, GameRng::from_seed(SEED));
        let mut second = GameSession::new(0.25, GameRng::from_seed(SEED));

        for step in 0..100 {
            assert_eq!(
                game_signature(&first),
                game_signature(&second),
                "seed {SEED} diverged before step {step}"
            );

            match first.interaction_state() {
                InteractionState::AwaitingPlayerAction => {
                    let action = first.snapshot().legal_actions[0];
                    assert_eq!(first.dispatch(action), second.dispatch(action));
                }
                InteractionState::Advancing => {
                    assert_eq!(first.advance(), second.advance());
                }
                InteractionState::Finished => return,
            }
        }

        panic!("seed {SEED} did not finish within 100 steps");
    }

    #[test]
    fn game_start_selects_an_active_player_and_enters_start_turn() {
        let mut game = configured_game(1);
        assert_eq!(game.state, GamePhase::NotStarted);
        let update = game.advance().unwrap();
        assert_eq!(game.state, GamePhase::StartTurn);
        assert!(matches!(update.events[0], GameEvent::GameStarted { .. }));
        assert!(matches!(
            game.active_player,
            ActivePlayer::Player | ActivePlayer::Cpu
        ));
        assert_eq!(game.interaction_state(), InteractionState::Advancing);
    }

    #[test]
    fn player_turn_rejects_unavailable_position_without_removing_card() {
        let mut game = configured_game(2);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        game.turn_announced = true;
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Blocked;
        game.board = Board::from_tiles(tiles);
        let card_id = game.human_player.hand[0].id;
        let before = game.human_player.hand.len();
        assert_eq!(
            game.dispatch(GameAction::new(card_id, Position::new(0, 0))),
            Err(GameError::IllegalPosition(Position::new(0, 0)))
        );
        assert_eq!(game.human_player.hand.len(), before);
        assert_eq!(game.state, GamePhase::StartTurn);
    }

    #[test]
    fn player_placement_enters_effect_processing_then_end_turn() {
        let mut game = configured_game(3);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        game.turn_announced = true;
        let card_id = game.human_player.hand[0].id;
        game.dispatch(GameAction::new(card_id, Position::new(0, 0)))
            .unwrap();
        assert_eq!(game.human_player.hand.len(), MAX_HAND_CARDS - 1);
        assert!(
            matches!(game.state, GamePhase::ApplyEffects { ref pending } if pending.is_empty())
        );
        assert_eq!(game.player_score(), 1);
        let turn_ended = game.advance().unwrap();
        assert!(matches!(turn_ended.events[0], GameEvent::TurnEnded { .. }));
        assert_eq!(game.state, GamePhase::EndTurn);
        let next_turn = game.advance().unwrap();
        assert!(matches!(next_turn.events[0], GameEvent::TurnStarted { .. }));
        assert_eq!(game.state, GamePhase::StartTurn);
        assert!(game.active_player == ActivePlayer::Cpu);
    }

    #[test]
    fn cpu_turn_chooses_an_available_position_and_consumes_one_card() {
        let mut game = configured_game(4);
        game.active_player = ActivePlayer::Cpu;
        game.state = GamePhase::StartTurn;
        let turn_started = game.advance().unwrap();
        assert!(matches!(
            turn_started.events[0],
            GameEvent::TurnStarted {
                player: PlayerSide::Cpu
            }
        ));
        let update = game.advance().unwrap();
        assert!(matches!(
            update.events[0],
            GameEvent::CardPlaced {
                player: PlayerSide::Cpu,
                ..
            }
        ));
        assert_eq!(game.cpu_player.hand.len(), MAX_HAND_CARDS - 1);
        assert_eq!(game.cpu_score(), 1);
        assert!(matches!(game.state, GamePhase::ApplyEffects { .. }));
    }

    #[test]
    fn capture_effect_changes_target_controller_one_effect_per_run() {
        let mut game = configured_game(5);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Occupied(BoardCard {
            controller: BoardSide::Blue,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            controller: BoardSide::Red,
            card: card(20, 0, BattleClass::Physical, 0, 0, 0),
        });
        game.board = Board::from_tiles(tiles);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::ApplyEffects {
            pending: VecDeque::from([PendingEffect::new(10, 20, Effect::DirectCapture)]),
        };
        let control_change = game.advance().unwrap();
        assert!(matches!(
            control_change.events[0],
            GameEvent::ControlChanged {
                card_id: 20,
                new_controller: BoardSide::Blue,
                ..
            }
        ));
        assert_eq!(
            game.board.get_card(Position::new(0, 1)).unwrap().controller,
            BoardSide::Blue
        );
        assert!(
            matches!(game.state, GamePhase::ApplyEffects { ref pending } if pending.is_empty())
        );
        let turn_ended = game.advance().unwrap();
        assert!(matches!(turn_ended.events[0], GameEvent::TurnEnded { .. }));
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
            controller: BoardSide::Blue,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            controller: BoardSide::Red,
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
            controller: BoardSide::Red,
            card: card(30, 0, BattleClass::Physical, 0, 0, 0),
        });
        game.board = Board::from_tiles(tiles);
        game.state = GamePhase::ApplyEffects {
            pending: VecDeque::new(),
        };
        game.apply_resolved_capture(ResolvedCapture {
            card_id: 20,
            new_controller: BoardSide::Blue,
            reason: ControlChangeReason::CombatVictory,
            combo_origin: Some(Position::new(0, 1)),
            ends_turn: false,
        })
        .unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 1)).unwrap().controller,
            BoardSide::Blue
        );
        assert!(
            matches!(game.state, GamePhase::ApplyEffects { ref pending } if pending.front().is_some_and(|effect| effect.target_card_id == 30))
        );

        game.apply_resolved_capture(ResolvedCapture {
            card_id: 10,
            new_controller: BoardSide::Red,
            reason: ControlChangeReason::CombatDefeat,
            combo_origin: None,
            ends_turn: true,
        })
        .unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 0)).unwrap().controller,
            BoardSide::Red
        );
        assert_eq!(game.state, GamePhase::EndTurn);
    }

    #[test]
    fn end_turn_finishes_for_full_board_or_two_empty_hands() {
        let mut game = configured_game(7);
        game.state = GamePhase::EndTurn;
        game.turn_ended_announced = true;
        game.human_player.hand.clear();
        game.cpu_player.hand.clear();
        let update = game.advance().unwrap();
        assert!(matches!(update.events[0], GameEvent::GameFinished { .. }));
        assert_eq!(game.state, GamePhase::Finished);

        let tiles = std::array::from_fn(|index| {
            Tile::Occupied(BoardCard {
                controller: BoardSide::Blue,
                card: card(100 + index as u64, 0, BattleClass::Physical, 0, 0, 0),
            })
        });
        game.board = Board::from_tiles(tiles);
        game.human_player
            .hand
            .push(card(1, 0, BattleClass::Physical, 0, 0, 0));
        game.state = GamePhase::EndTurn;
        game.turn_ended_announced = true;
        let update = game.advance().unwrap();
        assert!(matches!(update.events[0], GameEvent::GameFinished { .. }));
        assert_eq!(game.state, GamePhase::Finished);
    }

    #[test]
    fn automated_full_game_reaches_completion_and_preserves_card_totals() {
        let mut game = configured_game(8);
        let mut steps = 0;
        while game.interaction_state() != InteractionState::Finished {
            match game.interaction_state() {
                InteractionState::AwaitingPlayerAction => {
                    let action = game.snapshot().legal_actions[0];
                    game.dispatch(action).unwrap();
                }
                InteractionState::Advancing => {
                    game.advance().unwrap();
                }
                InteractionState::Finished => unreachable!(),
            }
            steps += 1;
            assert!(steps < 100);
        }
        assert!(game.human_player.hand.is_empty());
        assert!(game.cpu_player.hand.is_empty());
        assert_eq!(game.player_score() + game.cpu_score(), MAX_HAND_CARDS * 2);
        assert_eq!(game.board.count_empty(), 16 - MAX_HAND_CARDS * 2);
    }

    #[test]
    fn invalid_missing_card_does_not_change_board() {
        let mut game = configured_game(9);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::StartTurn;
        game.turn_announced = true;
        assert_eq!(
            game.dispatch(GameAction::new(999, Position::new(0, 0))),
            Err(GameError::CardNotInHand(999))
        );
        assert!(game.board.is_available(Position::new(0, 0)));
        assert_eq!(game.human_player.hand.len(), MAX_HAND_CARDS);
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
    fn combat_and_control_change_are_separate_event_transitions() {
        let mut game = configured_game(11);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Occupied(BoardCard {
            controller: BoardSide::Blue,
            card: card(10, 0, BattleClass::Physical, 100, 100, 100),
        });
        tiles[1] = Tile::Occupied(BoardCard {
            controller: BoardSide::Red,
            card: card(20, 0, BattleClass::Physical, 100, 100, 100),
        });
        game.board = Board::from_tiles(tiles);
        game.active_player = ActivePlayer::Player;
        game.state = GamePhase::ApplyEffects {
            pending: VecDeque::from([PendingEffect::new(10, 20, Effect::Attack)]),
        };

        let combat = game.advance().unwrap();
        let (captured_id, controller_before) = match combat.events[0] {
            GameEvent::CombatResolved {
                attacker_id: 10,
                defender_id: 20,
                outcome: CombatResult::AttackerWon,
            } => (20, BoardSide::Red),
            GameEvent::CombatResolved {
                attacker_id: 10,
                defender_id: 20,
                outcome: CombatResult::DefenderWon,
            } => (10, BoardSide::Blue),
            ref event => panic!("unexpected event: {event:?}"),
        };
        assert_eq!(
            game.board
                .position_of_card(captured_id)
                .unwrap()
                .0
                .controller,
            controller_before
        );

        let control_change = game.advance().unwrap();
        assert!(matches!(
            control_change.events[0],
            GameEvent::ControlChanged { card_id, .. } if card_id == captured_id
        ));
        assert_ne!(
            game.board
                .position_of_card(captured_id)
                .unwrap()
                .0
                .controller,
            controller_before
        );
    }

    #[test]
    fn generated_runtime_card_ids_are_unique_within_a_session() {
        for seed in 0..64 {
            let game = GameSession::new(0.0, GameRng::from_seed(seed));
            let ids = game
                .human_player
                .hand
                .iter()
                .chain(&game.cpu_player.hand)
                .map(|card| card.id)
                .collect::<HashSet<_>>();
            assert_eq!(ids.len(), MAX_HAND_CARDS * 2, "seed {seed}");
        }
    }
}
