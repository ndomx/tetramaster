use rand::{Rng, RngExt, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    assets::CARDS,
    commands::{
        AttackOutcome, AttackParams, GenerateEffectsParams, attack, generate_effects,
        spread_victory_effects,
    },
    models::{
        action::Action, active_player::ActivePlayer, board::Board, card::Card,
        effect_instance::EffectInstance, game_state::GameState, player::Player, position::Position,
    },
    utils::{constants::MAX_HAND_CARDS, random::VecRandomExt},
};

use super::effect::Effect;

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
        Ok(())
    }

    pub fn awaiting_input(&self) -> bool {
        self.state == GameState::StartTurn && self.active_player == ActivePlayer::Player
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
            Effect::Attack => self.attack(effect_instance),
            Effect::Capture => self.capture(effect_instance),
        }
    }

    fn end_turn(&mut self) -> TurnResult {
        if self.board.count_empty() == 0 {
            self.state = GameState::Finished;
            return Ok(());
        }

        if self.player.hand.is_empty() && self.cpu.hand.is_empty() {
            self.state = GameState::Finished;
            return Ok(());
        }

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
            AttackOutcome::Victory => self.on_victory(source_tc.owner_id, target_pos),
            AttackOutcome::Defeat => self.on_defeat(target_tc.owner_id, source_pos),
        }
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

    fn on_victory(&mut self, owner_id: u64, pos: Position) -> TurnResult {
        self.board.set_owner(pos, owner_id)?;

        let GameState::ApplyEffects { pending } = &mut self.state else {
            return Err("skip side effects".into());
        };

        spread_victory_effects(
            GenerateEffectsParams {
                position: pos,
                board: &self.board,
            },
            pending,
        )?;

        Ok(())
    }

    fn on_defeat(&mut self, owner_id: u64, pos: Position) -> TurnResult {
        self.state = GameState::EndTurn;
        self.board.set_owner(pos, owner_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{battle_class::BattleClass, effect::Effect, tile::Tile, tile_card::TileCard},
        test_support::{card, empty_tiles},
        utils::constants::MAX_HAND_CARDS,
    };
    use std::collections::VecDeque;

    fn configured_game<'a>(rng: &'a mut ThreadRng) -> Game<'a> {
        let mut game = Game::new(0.0, rng);
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
    fn game_start_selects_an_active_player_and_enters_start_turn() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        assert_eq!(game.state, GameState::NotStarted);
        game.run().unwrap();
        assert_eq!(game.state, GameState::StartTurn);
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
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        game.active_player = ActivePlayer::Player;
        game.state = GameState::StartTurn;
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Block;
        game.board = Board::from_tiles(tiles);
        let card_id = game.player.hand[0].id;
        let before = game.player.hand.len();
        assert!(
            game.play_card(Action::new(card_id, Position::new(0, 0)))
                .is_err()
        );
        assert_eq!(game.player.hand.len(), before);
        assert_eq!(game.state, GameState::StartTurn);
    }

    #[test]
    fn player_placement_enters_effect_processing_then_end_turn() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        game.active_player = ActivePlayer::Player;
        game.state = GameState::StartTurn;
        let card_id = game.player.hand[0].id;
        game.play_card(Action::new(card_id, Position::new(0, 0)))
            .unwrap();
        assert_eq!(game.player.hand.len(), MAX_HAND_CARDS - 1);
        assert!(
            matches!(game.state, GameState::ApplyEffects { ref pending } if pending.is_empty())
        );
        assert_eq!(game.player_score(), 1);
        game.run().unwrap();
        assert_eq!(game.state, GameState::EndTurn);
        game.run().unwrap();
        assert_eq!(game.state, GameState::StartTurn);
        assert!(game.active_player == ActivePlayer::Cpu);
    }

    #[test]
    fn cpu_turn_chooses_an_available_position_and_consumes_one_card() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        game.active_player = ActivePlayer::Cpu;
        game.state = GameState::StartTurn;
        game.run().unwrap();
        assert_eq!(game.cpu.hand.len(), MAX_HAND_CARDS - 1);
        assert_eq!(game.cpu_score(), 1);
        assert!(matches!(game.state, GameState::ApplyEffects { .. }));
    }

    #[test]
    fn capture_effect_changes_target_owner_one_effect_per_run() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Card(TileCard {
            owner_id: 1,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Card(TileCard {
            owner_id: 2,
            card: card(20, 0, BattleClass::Physical, 0, 0, 0),
        });
        game.board = Board::from_tiles(tiles);
        game.active_player = ActivePlayer::Player;
        game.state = GameState::ApplyEffects {
            pending: VecDeque::from([EffectInstance::new(10, 20, Effect::Capture)]),
        };
        game.run().unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 1)).unwrap().owner_id,
            1
        );
        assert!(
            matches!(game.state, GameState::ApplyEffects { ref pending } if pending.is_empty())
        );
        game.run().unwrap();
        assert_eq!(game.state, GameState::EndTurn);
    }

    #[test]
    fn victory_propagates_capture_while_defeat_flips_source_and_ends_turn() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        let mut tiles = empty_tiles();
        tiles[0] = Tile::Card(TileCard {
            owner_id: 1,
            card: card(10, 0, BattleClass::Physical, 0, 0, 0),
        });
        tiles[1] = Tile::Card(TileCard {
            owner_id: 2,
            card: card(
                20,
                1 << crate::models::direction::Direction::E as u8,
                BattleClass::Physical,
                0,
                0,
                0,
            ),
        });
        tiles[2] = Tile::Card(TileCard {
            owner_id: 2,
            card: card(30, 0, BattleClass::Physical, 0, 0, 0),
        });
        game.board = Board::from_tiles(tiles);
        game.state = GameState::ApplyEffects {
            pending: VecDeque::new(),
        };
        game.on_victory(1, Position::new(0, 1)).unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 1)).unwrap().owner_id,
            1
        );
        assert!(
            matches!(game.state, GameState::ApplyEffects { ref pending } if pending.front().is_some_and(|effect| effect.target_card_id == 30))
        );

        game.on_defeat(2, Position::new(0, 0)).unwrap();
        assert_eq!(
            game.board.get_card(Position::new(0, 0)).unwrap().owner_id,
            2
        );
        assert_eq!(game.state, GameState::EndTurn);
    }

    #[test]
    fn end_turn_finishes_for_full_board_or_two_empty_hands() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        game.state = GameState::EndTurn;
        game.player.hand.clear();
        game.cpu.hand.clear();
        game.run().unwrap();
        assert_eq!(game.state, GameState::Finished);

        let tiles = std::array::from_fn(|index| {
            Tile::Card(TileCard {
                owner_id: 1,
                card: card(100 + index as u64, 0, BattleClass::Physical, 0, 0, 0),
            })
        });
        game.board = Board::from_tiles(tiles);
        game.player
            .hand
            .push(card(1, 0, BattleClass::Physical, 0, 0, 0));
        game.state = GameState::EndTurn;
        game.run().unwrap();
        assert_eq!(game.state, GameState::Finished);
    }

    #[test]
    fn automated_full_game_reaches_completion_and_preserves_card_totals() {
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        game.active_player = ActivePlayer::Player;
        game.state = GameState::StartTurn;
        let mut steps = 0;
        while game.state != GameState::Finished {
            if game.awaiting_input() {
                let card_id = game.player.hand[0].id;
                let target = (0..16)
                    .filter_map(crate::utils::helpers::idx2pos)
                    .find(|position| game.board.is_available(*position))
                    .unwrap();
                game.play_card(Action::new(card_id, target)).unwrap();
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
        let mut rng = rand::rng();
        let mut game = configured_game(&mut rng);
        assert!(
            game.play_card(Action::new(999, Position::new(0, 0)))
                .is_err()
        );
        assert!(game.board.is_available(Position::new(0, 0)));
        assert_eq!(game.player.hand.len(), MAX_HAND_CARDS);
    }
}
