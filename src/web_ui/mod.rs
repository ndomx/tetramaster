mod components;

use dioxus::prelude::*;

use crate::{
    models::{
        core::geometry::Position,
        session::{
            CombatResult, ControlChangeReason, GameAction, GameError, GameEvent, GameResult,
            GameSession, GameSnapshot, GameUpdate, InteractionState, PlayerSide,
        },
    },
    utils::random::GameRng,
};

use components::GameApp;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

struct WebGame {
    session: GameSession,
    snapshot: GameSnapshot,
    interaction: InteractionState,
    selected_card_id: Option<u64>,
    status: String,
    error: Option<String>,
    restart_confirmation_open: bool,
}

impl WebGame {
    fn new() -> Self {
        let session = GameSession::new(GameRng::from_seed(rand::random()));
        let snapshot = session.snapshot();
        let interaction = session.interaction_state();

        Self {
            session,
            snapshot,
            interaction,
            selected_card_id: None,
            status: "Starting game…".into(),
            error: None,
            restart_confirmation_open: false,
        }
    }

    fn select_card(&mut self, card_id: u64) {
        if self.interaction != InteractionState::AwaitingPlayerAction
            || !self
                .snapshot
                .human_hand
                .iter()
                .any(|card| card.id == card_id)
        {
            return;
        }

        self.selected_card_id = (self.selected_card_id != Some(card_id)).then_some(card_id);
        self.error = None;
    }

    fn cancel_selection(&mut self) {
        self.selected_card_id = None;
        self.error = None;
    }

    fn handle_escape(&mut self) {
        if self.restart_confirmation_open {
            self.cancel_restart();
        } else {
            self.cancel_selection();
        }
    }

    fn play_selected_card(&mut self, position: Position) {
        let Some(action) = action_for_selection(&self.snapshot, self.selected_card_id, position)
        else {
            self.error = Some("Select a card and choose a legal board cell.".into());
            return;
        };

        match self.session.dispatch(action) {
            Ok(update) => {
                self.selected_card_id = None;
                self.apply_update(update);
            }
            Err(error) => self.show_error(error),
        }
    }

    fn advance_once(&mut self) {
        match self.session.advance() {
            Ok(update) => self.apply_update(update),
            Err(error) => self.show_error(error),
        }
    }

    fn request_restart(&mut self) {
        if self.interaction == InteractionState::Finished {
            *self = Self::new();
        } else {
            self.restart_confirmation_open = true;
        }
    }

    fn cancel_restart(&mut self) {
        self.restart_confirmation_open = false;
    }

    fn confirm_restart(&mut self) {
        *self = Self::new();
    }

    fn apply_update(&mut self, update: GameUpdate) {
        if let Some(event) = update.events.last() {
            self.status = event_message(event);
        }
        self.snapshot = update.snapshot;
        self.interaction = update.interaction;
        self.error = None;

        if self.selected_card_id.is_some_and(|selected| {
            !self
                .snapshot
                .human_hand
                .iter()
                .any(|card| card.id == selected)
        }) {
            self.selected_card_id = None;
        }
    }

    fn show_error(&mut self, error: GameError) {
        self.error = Some(format!("Unable to continue: {error}"));
    }
}

fn action_for_selection(
    snapshot: &GameSnapshot,
    selected_card_id: Option<u64>,
    position: Position,
) -> Option<GameAction> {
    let action = GameAction::new(selected_card_id?, position);
    snapshot.legal_actions.contains(&action).then_some(action)
}

#[component]
pub fn App() -> Element {
    let mut game = use_signal(WebGame::new);

    use_effect(move || {
        if game.read().interaction == InteractionState::Advancing {
            spawn(async move { game.write().advance_once() });
        }
    });

    let snapshot = game.read().snapshot.clone();
    let interaction = game.read().interaction;
    let selected_card_id = game.read().selected_card_id;
    let status = game.read().status.clone();
    let error = game.read().error.clone();
    let restart_confirmation_open = game.read().restart_confirmation_open;

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        GameApp {
            snapshot,
            interaction,
            selected_card_id,
            status,
            error,
            restart_confirmation_open,
            on_select_card: move |card_id| game.write().select_card(card_id),
            on_cancel_selection: move |_| game.write().cancel_selection(),
            on_play_card: move |position| game.write().play_selected_card(position),
            on_request_restart: move |_| game.write().request_restart(),
            on_cancel_restart: move |_| game.write().cancel_restart(),
            on_confirm_restart: move |_| game.write().confirm_restart(),
            on_key_down: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    game.write().handle_escape();
                }
            },
        }
    }
}

fn player_label(side: PlayerSide) -> &'static str {
    match side {
        PlayerSide::Human => "Player",
        PlayerSide::Cpu => "CPU",
    }
}

fn event_message(event: &GameEvent) -> String {
    match event {
        GameEvent::GameStarted { first_player } => {
            format!("Game started. {} goes first.", player_label(*first_player))
        }
        GameEvent::TurnStarted { player } => format!("{} turn.", player_label(*player)),
        GameEvent::CardPlaced {
            player, position, ..
        } => format!(
            "{} played at row {}, column {}.",
            player_label(*player),
            position.row + 1,
            position.col + 1
        ),
        GameEvent::CombatResolved { outcome, .. } => match outcome {
            CombatResult::AttackerWon => "Combat resolved: attacker won.".into(),
            CombatResult::DefenderWon => "Combat resolved: defender won.".into(),
        },
        GameEvent::ControlChanged { reason, .. } => {
            let reason = match reason {
                ControlChangeReason::DirectCapture => "direct capture",
                ControlChangeReason::CombatVictory => "combat victory",
                ControlChangeReason::CombatDefeat => "combat defeat",
                ControlChangeReason::Combo => "combo",
            };
            format!("Card control changed by {reason}.")
        }
        GameEvent::TurnEnded { player } => format!("{} turn ended.", player_label(*player)),
        GameEvent::GameFinished { result } => match result {
            GameResult::Winner(side) => format!("Game finished. {} won.", player_label(*side)),
            GameResult::Draw => "Game finished in a draw.".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_card_and_legal_cell_dispatch_through_the_session_contract() {
        let mut session = GameSession::new(GameRng::from_seed(0x2_03));
        while session.interaction_state() == InteractionState::Advancing {
            session.advance().unwrap();
        }
        let snapshot = session.snapshot();
        let selected_card_id = snapshot.human_hand.first().unwrap().id;
        let position = snapshot.legal_actions.first().unwrap().position();
        let action = action_for_selection(&snapshot, Some(selected_card_id), position).unwrap();
        let update = session.dispatch(action).unwrap();

        assert_eq!(update.interaction, InteractionState::Advancing);
        assert!(
            matches!(update.events.as_slice(), [GameEvent::CardPlaced { player: PlayerSide::Human, card_id, position: placed_position }] if *card_id == selected_card_id && *placed_position == position)
        );
    }

    #[test]
    fn selecting_the_same_card_twice_cancels_the_selection() {
        let mut game = WebGame::new();
        while game.interaction == InteractionState::Advancing {
            game.advance_once();
        }
        let card_id = game.snapshot.human_hand[0].id;
        game.select_card(card_id);
        assert_eq!(game.selected_card_id, Some(card_id));
        game.select_card(card_id);
        assert_eq!(game.selected_card_id, None);
    }

    #[test]
    fn restart_confirmation_preserves_game_until_confirmed() {
        let mut game = WebGame::new();
        let snapshot = game.snapshot.clone();
        game.request_restart();
        assert!(game.restart_confirmation_open);
        assert_eq!(game.snapshot, snapshot);
        game.cancel_restart();
        assert!(!game.restart_confirmation_open);
        assert_eq!(game.snapshot, snapshot);
    }

    #[test]
    fn escape_closes_restart_confirmation_before_clearing_selection() {
        let mut game = WebGame::new();
        while game.interaction == InteractionState::Advancing {
            game.advance_once();
        }
        let card_id = game.snapshot.human_hand[0].id;
        game.select_card(card_id);
        game.request_restart();

        game.handle_escape();

        assert!(!game.restart_confirmation_open);
        assert_eq!(game.selected_card_id, Some(card_id));

        game.handle_escape();
        assert_eq!(game.selected_card_id, None);
    }

    #[test]
    fn invalid_web_selection_reports_an_error_without_losing_the_snapshot() {
        let mut game = WebGame::new();
        while game.interaction == InteractionState::Advancing {
            game.advance_once();
        }
        let snapshot_before = game.snapshot.clone();
        game.play_selected_card(Position::new(0, 0));
        assert_eq!(game.snapshot, snapshot_before);
        assert_eq!(game.session.snapshot(), snapshot_before);
        assert_eq!(
            game.error.as_deref(),
            Some("Select a card and choose a legal board cell.")
        );
    }
}
