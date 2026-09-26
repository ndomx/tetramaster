use dioxus::prelude::*;

use crate::{
    models::{
        core::{board::BoardSide, geometry::Position},
        session::{
            BoardTileSnapshot, CardSnapshot, GameResult, GameSnapshot, InteractionState, PlayerSide,
        },
    },
    utils::constants::BOARD_SIZE,
};

use super::action_for_selection;

const CARD_ARTWORK: Asset = asset!("/assets/cards/fallback.png");

#[derive(Clone, Copy, PartialEq)]
enum CardSize {
    Board,
    Hand,
}

#[derive(Clone, Copy, PartialEq)]
enum CardOwner {
    Human,
    Cpu,
}

#[component]
pub(super) fn GameApp(
    snapshot: GameSnapshot,
    interaction: InteractionState,
    selected_card_id: Option<u64>,
    status: String,
    error: Option<String>,
    restart_confirmation_open: bool,
    on_select_card: EventHandler<u64>,
    on_cancel_selection: EventHandler<MouseEvent>,
    on_play_card: EventHandler<Position>,
    on_request_restart: EventHandler<MouseEvent>,
    on_cancel_restart: EventHandler<MouseEvent>,
    on_confirm_restart: EventHandler<MouseEvent>,
    on_key_down: EventHandler<KeyboardEvent>,
) -> Element {
    rsx! {
        main {
            onkeydown: move |event| on_key_down.call(event),
            class: "min-h-screen overflow-x-hidden bg-[radial-gradient(circle_at_top,#24365d_0%,#111827_42%,#050914_100%)] px-5 py-5 text-stone-100 lg:px-8 lg:py-6",
            div {
                class: "mx-auto flex min-h-[calc(100vh-2.5rem)] max-w-[1380px] flex-col",
                header {
                    class: "flex items-center justify-between border-b border-amber-200/25 pb-3",
                    div {
                        p { class: "text-xs font-semibold uppercase tracking-[0.32em] text-amber-200/70", "Card battle" }
                        h1 { class: "font-serif text-3xl font-black tracking-wide text-amber-100 drop-shadow", "Tetra Master" }
                    }
                    button {
                        r#type: "button",
                        class: "rounded-md border border-amber-200/50 bg-slate-950/50 px-4 py-2 text-sm font-bold text-amber-100 transition hover:border-amber-100 hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-amber-200",
                        onclick: move |event| on_request_restart.call(event),
                        "New game"
                    }
                }

                div {
                    class: "grid flex-1 grid-cols-[minmax(180px,0.7fr)_minmax(380px,1.4fr)_minmax(180px,0.7fr)] items-center gap-5 py-3",
                    aside {
                        class: "flex h-full min-h-0 flex-col justify-center gap-4",
                        Score { snapshot: snapshot.clone() }
                        GameStatus { interaction, status, error }
                    }
                    Board { snapshot: snapshot.clone(), selected_card_id, on_play_card }
                    aside {
                        class: "flex h-full flex-col justify-center gap-4",
                        div {
                            class: "rounded-xl border border-white/10 bg-slate-950/45 p-4 shadow-xl backdrop-blur-sm",
                            h2 { class: "font-serif text-lg font-bold text-amber-100", "How to play" }
                            ol {
                                class: "mt-3 space-y-3 text-sm leading-relaxed text-slate-300",
                                li { class: "flex gap-3", span { class: "font-bold text-amber-300", "1" } "Choose a card from your hand." }
                                li { class: "flex gap-3", span { class: "font-bold text-amber-300", "2" } "Play it on a marked open cell." }
                                li { class: "flex gap-3", span { class: "font-bold text-amber-300", "3" } "Arrows determine neighboring interactions." }
                            }
                        }
                        div {
                            class: "rounded-xl border border-white/10 bg-slate-950/45 p-4 text-xs text-slate-400",
                            p { class: "font-bold uppercase tracking-widest text-slate-300", "Card stats" }
                            p { class: "mt-2", "Attack · Class · Physical defense · Magic defense" }
                            p { class: "mt-2", "P Physical · M Magic · X Flexible · A Assault" }
                        }
                    }
                }

                Hand {
                    cards: snapshot.human_hand.clone(),
                    interaction,
                    selected_card_id,
                    on_select_card,
                    on_cancel_selection,
                }
            }

            if let Some(result) = snapshot.result {
                GameOverDialog {
                    result,
                    human_score: snapshot.human_score,
                    cpu_score: snapshot.cpu_score,
                    on_play_again: on_request_restart,
                }
            } else if restart_confirmation_open {
                RestartDialog { on_cancel: on_cancel_restart, on_confirm: on_confirm_restart }
            }
        }
    }
}

#[component]
fn Score(snapshot: GameSnapshot) -> Element {
    rsx! {
        section {
            aria_label: "Score",
            class: "overflow-hidden rounded-xl border border-amber-200/25 bg-slate-950/55 shadow-xl backdrop-blur-sm",
            h2 { class: "border-b border-white/10 px-4 py-3 text-xs font-bold uppercase tracking-[0.24em] text-amber-200/70", "Score" }
            div {
                class: "grid grid-cols-2 divide-x divide-white/10",
                div { class: "p-4 text-center", p { class: "text-xs font-bold uppercase tracking-wider text-blue-300", "◆ Player" } p { class: "mt-1 font-serif text-4xl font-black text-blue-100", "{snapshot.human_score}" } }
                div { class: "p-4 text-center", p { class: "text-xs font-bold uppercase tracking-wider text-rose-300", "CPU ▲" } p { class: "mt-1 font-serif text-4xl font-black text-rose-100", "{snapshot.cpu_score}" } }
            }
            p { class: "border-t border-white/10 px-4 py-2 text-center text-xs text-slate-400", "CPU hand: {snapshot.cpu_hand_count} cards" }
        }
    }
}

#[component]
fn GameStatus(interaction: InteractionState, status: String, error: Option<String>) -> Element {
    let (label, marker) = match interaction {
        InteractionState::AwaitingPlayerAction => ("Your turn", "◆"),
        InteractionState::Advancing => ("Resolving", "◌"),
        InteractionState::Finished => ("Match complete", "■"),
    };
    rsx! {
        section {
            aria_live: "polite",
            aria_label: "Game status",
            class: "min-h-32 rounded-xl border border-white/10 bg-slate-950/45 p-4 shadow-xl backdrop-blur-sm",
            p { class: "text-xs font-bold uppercase tracking-[0.2em] text-amber-200", "{marker} {label}" }
            p { class: "mt-3 text-sm leading-relaxed text-slate-200", "{status}" }
            if let Some(error) = error { p { role: "alert", class: "mt-3 border-l-2 border-rose-400 pl-3 text-sm font-semibold text-rose-200", "{error}" } }
        }
    }
}

#[component]
fn Board(
    snapshot: GameSnapshot,
    selected_card_id: Option<u64>,
    on_play_card: EventHandler<Position>,
) -> Element {
    rsx! {
        section {
            aria_label: "Game board",
            class: "mx-auto w-[min(48vw,47vh)] min-w-[360px] max-w-[520px] rounded-2xl border border-amber-200/30 bg-[linear-gradient(135deg,rgba(120,53,15,0.3),rgba(15,23,42,0.92))] p-3 shadow-2xl shadow-black/50",
            div {
                class: "grid grid-cols-4 gap-2",
                for (index, tile) in snapshot.board.iter().enumerate() {
                    {
                        let position = Position::new(index / BOARD_SIZE, index % BOARD_SIZE);
                        let is_legal = action_for_selection(&snapshot, selected_card_id, position).is_some();
                        rsx! { BoardCell { key: "{index}", tile: tile.clone(), position, is_legal, on_play_card } }
                    }
                }
            }
        }
    }
}

#[component]
fn BoardCell(
    tile: BoardTileSnapshot,
    position: Position,
    is_legal: bool,
    on_play_card: EventHandler<Position>,
) -> Element {
    let row = position.row + 1;
    let column = position.col + 1;
    match tile {
        BoardTileSnapshot::Empty => {
            let class = if is_legal {
                "group relative aspect-[84/102] overflow-hidden rounded-lg border-2 border-dashed border-emerald-300 bg-emerald-950/50 p-2 text-emerald-100 transition hover:border-emerald-100 hover:bg-emerald-900/70 focus:outline-none focus:ring-4 focus:ring-emerald-200/70"
            } else {
                "relative aspect-[84/102] overflow-hidden rounded-lg border border-slate-600/60 bg-slate-950/45 p-2 text-slate-500"
            };
            let label = if is_legal {
                "Available — play card"
            } else {
                "Empty"
            };
            rsx! {
                button {
                    r#type: "button", class, disabled: !is_legal,
                    aria_label: "Row {row}, column {column}: {label}",
                    onclick: move |_| on_play_card.call(position),
                    span { class: "absolute left-2 top-2 font-mono text-[10px] opacity-70", "{row}·{column}" }
                    span { class: "flex h-full items-center justify-center text-center text-xs font-bold uppercase tracking-wider", if is_legal { "+ Play" } else { "Empty" } }
                }
            }
        }
        BoardTileSnapshot::Blocked => rsx! {
            div {
                role: "img", aria_label: "Row {row}, column {column}: Blocked",
                class: "relative aspect-[84/102] overflow-hidden rounded-lg border border-amber-700/70 bg-[repeating-linear-gradient(135deg,rgba(120,53,15,0.45)_0,rgba(120,53,15,0.45)_8px,rgba(30,41,59,0.65)_8px,rgba(30,41,59,0.65)_16px)] p-2 text-amber-200",
                span { class: "absolute left-2 top-2 font-mono text-[10px] opacity-70", "{row}·{column}" }
                span { class: "flex h-full items-center justify-center text-2xl", "╳" }
                span { class: "absolute inset-x-0 bottom-3 text-center text-[10px] font-bold uppercase tracking-widest", "Blocked" }
            }
        },
        BoardTileSnapshot::Occupied { controller, card } => {
            let owner = match controller {
                BoardSide::Blue => CardOwner::Human,
                BoardSide::Red => CardOwner::Cpu,
            };
            rsx! { div { class: "aspect-[84/102]", aria_label: "Row {row}, column {column}", CardView { card, size: CardSize::Board, owner, selected: false } } }
        }
    }
}

#[component]
fn Hand(
    cards: Vec<CardSnapshot>,
    interaction: InteractionState,
    selected_card_id: Option<u64>,
    on_select_card: EventHandler<u64>,
    on_cancel_selection: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        section {
            aria_label: "Your hand", class: "border-t border-amber-200/20 pt-3",
            div {
                class: "mb-2 flex items-center justify-between",
                div { class: "flex items-baseline gap-3", h2 { class: "font-serif text-lg font-bold text-amber-100", "Your hand" } p { class: "text-xs text-slate-400", "Choose a card, then an available cell." } }
                if selected_card_id.is_some() { button { r#type: "button", class: "rounded border border-slate-500 px-3 py-1 text-xs font-bold text-slate-200 hover:border-slate-300 focus:outline-none focus:ring-2 focus:ring-amber-200", onclick: move |event| on_cancel_selection.call(event), "Cancel selection" } }
            }
            div {
                class: "mx-auto grid max-w-[760px] grid-cols-5 gap-3",
                for card in cards {
                    {
                        let card_id = card.id;
                        let selected = selected_card_id == Some(card_id);
                        let accessible_name = if selected {
                            format!("{} selected. Press again to cancel.", card.name)
                        } else {
                            format!("Select {}", card.name)
                        };
                        rsx! {
                            button {
                                key: "{card_id}", r#type: "button",
                                class: "min-w-0 rounded-xl focus:outline-none focus:ring-4 focus:ring-amber-200/80 disabled:cursor-not-allowed disabled:opacity-50",
                                disabled: interaction != InteractionState::AwaitingPlayerAction,
                                aria_pressed: selected,
                                aria_label: accessible_name,
                                onclick: move |_| on_select_card.call(card_id),
                                CardView { card, size: CardSize::Hand, owner: CardOwner::Human, selected }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CardView(card: CardSnapshot, size: CardSize, owner: CardOwner, selected: bool) -> Element {
    let owner_class = match owner {
        CardOwner::Human => "border-blue-300 bg-gradient-to-b from-blue-700 to-blue-950",
        CardOwner::Cpu => "border-rose-300 bg-gradient-to-b from-rose-700 to-rose-950",
    };
    let owner_label = match owner {
        CardOwner::Human => "◆ Player",
        CardOwner::Cpu => "CPU ▲",
    };
    let selected_class = if selected {
        "-translate-y-2 ring-4 ring-amber-200 shadow-amber-300/30"
    } else {
        "ring-1 ring-black/30"
    };
    let padding = match size {
        CardSize::Board => "p-1.5",
        CardSize::Hand => "p-2",
    };
    let image_height = match size {
        CardSize::Board => "h-[48%]",
        CardSize::Hand => "h-16",
    };
    let name_size = match size {
        CardSize::Board => "text-[10px]",
        CardSize::Hand => "text-xs",
    };
    let class = format!(
        "relative h-full w-full overflow-hidden rounded-lg border-2 {owner_class} {selected_class} {padding} text-left shadow-lg transition"
    );
    rsx! {
        article {
            class,
            div {
                class: "pointer-events-none absolute inset-0",
                for arrow in arrow_markers(card.arrows) { span { class: "absolute z-10 flex h-4 w-4 items-center justify-center rounded-full bg-amber-100 text-[9px] font-black text-slate-950 shadow {arrow.class}", aria_hidden: "true", "{arrow.symbol}" } }
            }
            div {
                class: "flex h-full flex-col",
                img { src: CARD_ARTWORK, alt: "", width: "84", height: "102", class: "{image_height} w-full rounded object-cover opacity-90" }
                h3 { class: "{name_size} mt-1 min-h-[2.35em] overflow-hidden text-center font-bold leading-tight text-white", title: "{card.name}", "{card.name}" }
                div { class: "mt-auto rounded bg-black/55 px-1 py-0.5 text-center font-mono text-[11px] font-black tracking-wider text-amber-100", aria_label: "Attack {card.stats.attack:X}, class {card.stats.battle_class}, physical defense {card.stats.phys_defense:X}, magic defense {card.stats.mag_defense:X}", "{card.stats.attack:X}{card.stats.battle_class}{card.stats.phys_defense:X}{card.stats.mag_defense:X}" }
                p { class: "mt-1 text-center text-[8px] font-black uppercase tracking-wider text-white/80", "{owner_label}" }
            }
        }
    }
}

struct ArrowMarker {
    symbol: &'static str,
    class: &'static str,
}

fn arrow_markers(mask: u8) -> Vec<ArrowMarker> {
    const ARROWS: [(&str, &str); 8] = [
        ("↑", "left-1/2 top-0 -translate-x-1/2 -translate-y-1/3"),
        ("↗", "right-0 top-0 translate-x-1/3 -translate-y-1/3"),
        ("→", "right-0 top-1/2 translate-x-1/3 -translate-y-1/2"),
        ("↘", "bottom-0 right-0 translate-x-1/3 translate-y-1/3"),
        ("↓", "bottom-0 left-1/2 -translate-x-1/2 translate-y-1/3"),
        ("↙", "bottom-0 left-0 -translate-x-1/3 translate-y-1/3"),
        ("←", "left-0 top-1/2 -translate-x-1/3 -translate-y-1/2"),
        ("↖", "left-0 top-0 -translate-x-1/3 -translate-y-1/3"),
    ];
    ARROWS
        .iter()
        .enumerate()
        .filter(|(index, _)| mask & (1 << index) != 0)
        .map(|(_, (symbol, class))| ArrowMarker { symbol, class })
        .collect()
}

#[component]
fn GameOverDialog(
    result: GameResult,
    human_score: usize,
    cpu_score: usize,
    on_play_again: EventHandler<MouseEvent>,
) -> Element {
    let (eyebrow, title) = match result {
        GameResult::Winner(PlayerSide::Human) => ("Victory", "You won the match"),
        GameResult::Winner(PlayerSide::Cpu) => ("Defeat", "The CPU won the match"),
        GameResult::Draw => ("Draw", "The match ended evenly"),
    };
    rsx! {
        div { role: "dialog", aria_modal: "true", aria_labelledby: "game-over-title", class: "fixed inset-0 z-50 flex items-center justify-center bg-slate-950/75 p-6 backdrop-blur-sm",
            div { class: "w-full max-w-md rounded-2xl border border-amber-200/50 bg-slate-900 p-8 text-center shadow-2xl shadow-black",
                p { class: "text-xs font-black uppercase tracking-[0.35em] text-amber-300", "{eyebrow}" }
                h2 { id: "game-over-title", class: "mt-3 font-serif text-3xl font-black text-amber-100", "{title}" }
                p { class: "mt-5 text-lg text-slate-200", "Player {human_score} — {cpu_score} CPU" }
                button { autofocus: true, r#type: "button", class: "mt-7 w-full rounded-lg border border-amber-200 bg-amber-200 px-5 py-3 font-black text-slate-950 transition hover:bg-amber-100 focus:outline-none focus:ring-4 focus:ring-amber-100/60", onclick: move |event| on_play_again.call(event), "Play again" }
            }
        }
    }
}

#[component]
fn RestartDialog(
    on_cancel: EventHandler<MouseEvent>,
    on_confirm: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { role: "dialog", aria_modal: "true", aria_labelledby: "restart-title", class: "fixed inset-0 z-50 flex items-center justify-center bg-slate-950/75 p-6 backdrop-blur-sm",
            div { class: "w-full max-w-md rounded-2xl border border-amber-200/50 bg-slate-900 p-8 shadow-2xl shadow-black",
                h2 { id: "restart-title", class: "font-serif text-2xl font-black text-amber-100", "Start a new game?" }
                p { class: "mt-3 text-sm leading-relaxed text-slate-300", "Your current match will be discarded." }
                div { class: "mt-7 flex justify-end gap-3",
                    button { autofocus: true, r#type: "button", class: "rounded-lg border border-slate-500 px-5 py-2 font-bold text-slate-100 hover:border-slate-300 focus:outline-none focus:ring-2 focus:ring-slate-200", onclick: move |event| on_cancel.call(event), "Keep playing" }
                    button { r#type: "button", class: "rounded-lg border border-amber-200 bg-amber-200 px-5 py-2 font-black text-slate-950 hover:bg-amber-100 focus:outline-none focus:ring-4 focus:ring-amber-100/60", onclick: move |event| on_confirm.call(event), "Start new game" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_markers_follow_all_eight_mask_bits() {
        assert!(arrow_markers(0).is_empty());
        assert_eq!(arrow_markers(0b0000_0001)[0].symbol, "↑");
        assert_eq!(arrow_markers(0b1000_0000)[0].symbol, "↖");
        assert_eq!(arrow_markers(u8::MAX).len(), 8);
    }
}
