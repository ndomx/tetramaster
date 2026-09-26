use dioxus::prelude::*;

use crate::{
    models::{
        core::{board::BoardSide, geometry::Position},
        session::{
            BoardTileSnapshot, CombatResult, ControlChangeReason, GameAction, GameError, GameEvent,
            GameResult, GameSession, GameSnapshot, GameUpdate, InteractionState, PlayerSide,
        },
    },
    utils::{constants::BOARD_SIZE, random::GameRng},
};

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

struct WebGame {
    session: GameSession,
    snapshot: GameSnapshot,
    interaction: InteractionState,
    selected_card_id: Option<u64>,
    status: String,
    error: Option<String>,
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

        self.selected_card_id = Some(card_id);
        self.error = None;
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
        let should_advance = game.read().interaction == InteractionState::Advancing;
        if should_advance {
            spawn(async move {
                game.write().advance_once();
            });
        }
    });

    let snapshot = game.read().snapshot.clone();
    let interaction = game.read().interaction;
    let selected_card_id = game.read().selected_card_id;
    let status = game.read().status.clone();
    let error = game.read().error.clone();

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main {
            class: "min-h-screen bg-slate-950 px-6 py-8 text-slate-100",
            div {
                class: "mx-auto flex max-w-6xl flex-col gap-6",
                header {
                    class: "flex items-end justify-between border-b border-slate-700 pb-4",
                    div {
                        h1 { class: "text-3xl font-bold tracking-wide", "Tetra Master" }
                        p { class: "mt-1 text-sm text-slate-400", "Playable vertical slice" }
                    }
                    div {
                        class: "text-right",
                        p { class: "text-xl font-semibold", "Player {snapshot.human_score} — {snapshot.cpu_score} CPU" }
                        p { class: "text-sm text-slate-400", "CPU cards: {snapshot.cpu_hand_count}" }
                    }
                }

                section {
                    aria_label: "Game status",
                    class: "min-h-20 rounded-lg border border-slate-700 bg-slate-900 p-4",
                    p { class: "font-medium", "{status}" }
                    p { class: "mt-1 text-sm text-slate-400", "{interaction_label(interaction)}" }
                    if let Some(error) = error {
                        p { role: "alert", class: "mt-2 text-sm font-semibold text-rose-400", "{error}" }
                    }
                }

                section {
                    aria_label: "Game board",
                    class: "mx-auto grid w-full max-w-2xl grid-cols-4 gap-2",
                    for (index, tile) in snapshot.board.iter().enumerate() {
                        {
                            let position = Position::new(index / BOARD_SIZE, index % BOARD_SIZE);
                            let is_legal = action_for_selection(&snapshot, selected_card_id, position).is_some();
                            let tile_class = board_tile_class(tile, is_legal);
                            let tile_label = board_tile_label(tile, is_legal);
                            rsx! {
                                button {
                                    key: "{index}",
                                    r#type: "button",
                                    class: "{tile_class}",
                                    disabled: !is_legal,
                                    aria_label: "Row {position.row + 1}, column {position.col + 1}: {tile_label}",
                                    onclick: move |_| game.write().play_selected_card(position),
                                    "{tile_label}"
                                }
                            }
                        }
                    }
                }

                section {
                    aria_label: "Your hand",
                    div {
                        class: "mb-3 flex items-center justify-between",
                        h2 { class: "text-lg font-semibold", "Your hand" }
                        p { class: "text-sm text-slate-400", "Select a card, then choose a highlighted cell." }
                    }
                    div {
                        class: "grid grid-cols-5 gap-3",
                        for card in &snapshot.human_hand {
                            {
                                let card_id = card.id;
                                let is_selected = selected_card_id == Some(card_id);
                                let class = hand_card_class(is_selected);
                                rsx! {
                                    button {
                                        key: "{card_id}",
                                        r#type: "button",
                                        class: "{class}",
                                        disabled: interaction != InteractionState::AwaitingPlayerAction,
                                        aria_pressed: is_selected,
                                        onclick: move |_| game.write().select_card(card_id),
                                        strong { class: "block truncate", title: "{card.name}", "{card.name}" }
                                        span {
                                            class: "mt-2 block text-xs text-blue-100",
                                            "ATK {card.stats.attack:X} · PDEF {card.stats.phys_defense:X} · MDEF {card.stats.mag_defense:X}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn board_tile_class(tile: &BoardTileSnapshot, is_legal: bool) -> &'static str {
    match tile {
        BoardTileSnapshot::Empty if is_legal => {
            "aspect-[4/5] min-h-24 rounded-lg border-2 border-emerald-400 bg-emerald-950 p-2 text-sm font-semibold text-emerald-200 transition hover:bg-emerald-900 focus:outline-none focus:ring-2 focus:ring-emerald-300"
        }
        BoardTileSnapshot::Empty => {
            "aspect-[4/5] min-h-24 rounded-lg border border-slate-700 bg-slate-900 p-2 text-sm font-semibold text-slate-600"
        }
        BoardTileSnapshot::Blocked => {
            "aspect-[4/5] min-h-24 rounded-lg border border-amber-700 bg-amber-950/70 p-2 text-sm font-semibold text-amber-300"
        }
        BoardTileSnapshot::Occupied {
            controller: BoardSide::Blue,
            ..
        } => {
            "aspect-[4/5] min-h-24 rounded-lg border border-blue-400 bg-blue-950 p-2 text-sm font-semibold text-blue-100"
        }
        BoardTileSnapshot::Occupied {
            controller: BoardSide::Red,
            ..
        } => {
            "aspect-[4/5] min-h-24 rounded-lg border border-rose-400 bg-rose-950 p-2 text-sm font-semibold text-rose-100"
        }
    }
}

fn board_tile_label(tile: &BoardTileSnapshot, is_legal: bool) -> String {
    match tile {
        BoardTileSnapshot::Empty if is_legal => "Play".into(),
        BoardTileSnapshot::Empty => "Empty".into(),
        BoardTileSnapshot::Blocked => "Blocked".into(),
        BoardTileSnapshot::Occupied { controller, card } => {
            format!("{}\n{}", controller_label(*controller), card.name)
        }
    }
}

fn hand_card_class(is_selected: bool) -> &'static str {
    if is_selected {
        "min-h-28 rounded-lg border-2 border-cyan-300 bg-blue-800 p-3 text-left shadow-lg shadow-cyan-950 focus:outline-none focus:ring-2 focus:ring-cyan-200"
    } else {
        "min-h-28 rounded-lg border border-blue-500 bg-blue-950 p-3 text-left hover:bg-blue-900 focus:outline-none focus:ring-2 focus:ring-blue-300 disabled:cursor-not-allowed disabled:opacity-60"
    }
}

fn interaction_label(interaction: InteractionState) -> &'static str {
    match interaction {
        InteractionState::AwaitingPlayerAction => "Your turn",
        InteractionState::Advancing => "Resolving game events…",
        InteractionState::Finished => "Match complete",
    }
}

fn player_label(side: PlayerSide) -> &'static str {
    match side {
        PlayerSide::Human => "Player",
        PlayerSide::Cpu => "CPU",
    }
}

fn controller_label(side: BoardSide) -> &'static str {
    match side {
        BoardSide::Blue => "Player",
        BoardSide::Red => "CPU",
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
        assert_eq!(
            session.interaction_state(),
            InteractionState::AwaitingPlayerAction
        );

        let snapshot = session.snapshot();
        let selected_card_id = snapshot.human_hand.first().unwrap().id;
        let position = snapshot.legal_actions.first().unwrap().position();
        let action = action_for_selection(&snapshot, Some(selected_card_id), position).unwrap();

        let update = session.dispatch(action).unwrap();

        assert_eq!(update.interaction, InteractionState::Advancing);
        assert!(matches!(
            update.events.as_slice(),
            [GameEvent::CardPlaced {
                player: PlayerSide::Human,
                card_id,
                position: placed_position,
            }] if *card_id == selected_card_id && *placed_position == position
        ));
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
