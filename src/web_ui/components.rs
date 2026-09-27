use dioxus::prelude::*;

use crate::{
    assets::artwork::{ArtworkLoadState, ArtworkResolver},
    models::{
        core::{board::BoardSide, geometry::Position},
        session::{
            BoardTileSnapshot, CardSnapshot, GameResult, GameSnapshot, InteractionState, PlayerSide,
        },
    },
    utils::constants::BOARD_SIZE,
};

use super::{CombatPresentation, PreviewKind, PreviewTarget, action_for_selection};

const FALLBACK_CARD_ARTWORK: Asset = asset!("/assets/cards/fallback.png");
const CUSTOM_CARD_ARTWORK: Asset = asset!("/assets/cards/custom");

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
    help_open: bool,
    previewed_position: Option<Position>,
    preview_targets: Vec<PreviewTarget>,
    combat: Option<CombatPresentation>,
    on_select_card: EventHandler<u64>,
    on_cancel_selection: EventHandler<MouseEvent>,
    on_play_card: EventHandler<Position>,
    on_preview_position: EventHandler<Position>,
    on_clear_preview: EventHandler<Position>,
    on_request_restart: EventHandler<MouseEvent>,
    on_cancel_restart: EventHandler<MouseEvent>,
    on_confirm_restart: EventHandler<MouseEvent>,
    on_show_help: EventHandler<MouseEvent>,
    on_hide_help: EventHandler<MouseEvent>,
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
                    div {
                        class: "flex items-center gap-2",
                        button {
                            r#type: "button",
                            aria_label: "How to play",
                            title: "How to play",
                            class: "flex h-10 w-10 items-center justify-center rounded-full border border-amber-200/50 bg-slate-950/50 text-lg font-black text-amber-100 transition hover:border-amber-100 hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-amber-200",
                            onclick: move |event| on_show_help.call(event),
                            "?"
                        }
                        button {
                            r#type: "button",
                            class: "rounded-md border border-amber-200/50 bg-slate-950/50 px-4 py-2 text-sm font-bold text-amber-100 transition hover:border-amber-100 hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-amber-200",
                            onclick: move |event| on_request_restart.call(event),
                            "New game"
                        }
                    }
                }

                OpponentHand { count: snapshot.cpu_hand_count }

                div {
                    class: "relative flex flex-1 items-center justify-center py-2",
                    Score { snapshot: snapshot.clone() }
                    div {
                        class: "flex flex-col items-center gap-2",
                        GameStatus { interaction, status, error }
                        Board {
                            snapshot: snapshot.clone(),
                            selected_card_id,
                            previewed_position,
                            preview_targets,
                            combat,
                            on_play_card,
                            on_preview_position,
                            on_clear_preview,
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
            } else if help_open {
                HelpDialog { on_close: on_hide_help }
            }
        }
    }
}

#[component]
fn Score(snapshot: GameSnapshot) -> Element {
    rsx! {
        section {
            aria_label: "Score",
            class: "absolute left-[8%] top-1/2 flex -translate-y-1/2 flex-col items-center gap-1 font-serif italic drop-shadow-lg",
            span { class: "text-xs font-black uppercase tracking-widest text-rose-300", "CPU ▲" }
            strong { class: "text-5xl font-black text-rose-300", "{snapshot.cpu_score}" }
            span { class: "my-1 h-px w-10 -rotate-12 bg-amber-200/60" }
            strong { class: "text-5xl font-black text-blue-200", "{snapshot.human_score}" }
            span { class: "text-xs font-black uppercase tracking-widest text-blue-300", "◆ Player" }
        }
    }
}

#[component]
fn OpponentHand(count: usize) -> Element {
    rsx! {
        section {
            aria_label: "Opponent hand: {count} hidden cards",
            class: "flex min-h-16 items-center justify-center gap-2 py-2",
            for index in 0..count {
                div {
                    key: "{index}",
                    role: "img",
                    aria_label: "Hidden opponent card {index + 1}",
                    class: "relative h-14 w-12 overflow-hidden rounded border-2 border-rose-300 bg-rose-950 shadow-lg",
                    img { src: FALLBACK_CARD_ARTWORK, alt: "", class: "absolute inset-0 h-full w-full object-cover opacity-75" }
                    span { class: "absolute inset-0 flex items-center justify-center bg-rose-950/30 text-xs font-black text-white", "▲" }
                }
            }
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
            class: "flex min-h-7 max-w-[520px] items-center gap-3 rounded-full border border-white/10 bg-slate-950/55 px-4 py-1 text-xs shadow backdrop-blur-sm",
            p { class: "shrink-0 font-bold uppercase tracking-[0.16em] text-amber-200", "{marker} {label}" }
            p { class: "truncate text-slate-300", "{status}" }
            if let Some(error) = error { p { role: "alert", class: "truncate font-semibold text-rose-200", "{error}" } }
        }
    }
}

#[component]
fn Board(
    snapshot: GameSnapshot,
    selected_card_id: Option<u64>,
    previewed_position: Option<Position>,
    preview_targets: Vec<PreviewTarget>,
    combat: Option<CombatPresentation>,
    on_play_card: EventHandler<Position>,
    on_preview_position: EventHandler<Position>,
    on_clear_preview: EventHandler<Position>,
) -> Element {
    rsx! {
        section {
            aria_label: "Game board",
            class: "mx-auto w-[min(48vw,40vh)] min-w-[320px] max-w-[520px] rounded-2xl border border-amber-200/30 bg-[linear-gradient(135deg,rgba(120,53,15,0.3),rgba(15,23,42,0.92))] p-3 shadow-2xl shadow-black/50",
            div {
                class: "grid grid-cols-4 gap-2",
                for (index, tile) in snapshot.board.iter().enumerate() {
                    {
                        let position = Position::new(index / BOARD_SIZE, index % BOARD_SIZE);
                        let is_legal = action_for_selection(&snapshot, selected_card_id, position).is_some();
                        rsx! { BoardCell {
                            key: "{index}",
                            tile: tile.clone(),
                            position,
                            is_legal,
                            is_previewed: previewed_position == Some(position),
                            preview_targets: preview_targets.clone(),
                            combat,
                            on_play_card,
                            on_preview_position,
                            on_clear_preview,
                        } }
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
    is_previewed: bool,
    preview_targets: Vec<PreviewTarget>,
    combat: Option<CombatPresentation>,
    on_play_card: EventHandler<Position>,
    on_preview_position: EventHandler<Position>,
    on_clear_preview: EventHandler<Position>,
) -> Element {
    let row = position.row + 1;
    let column = position.col + 1;
    match tile {
        BoardTileSnapshot::Empty => {
            let class = if is_previewed {
                "relative aspect-[84/102] overflow-hidden rounded-lg border-2 border-solid border-amber-100 bg-amber-300/35 shadow-[inset_0_0_28px_rgba(253,230,138,0.55)] transition focus:outline-none focus:ring-4 focus:ring-amber-200/80"
            } else if is_legal {
                "relative aspect-[84/102] overflow-hidden rounded-lg border-2 border-dashed border-emerald-300 bg-transparent transition hover:border-emerald-100 focus:outline-none focus:ring-4 focus:ring-emerald-200/70"
            } else {
                "relative aspect-[84/102] rounded-lg border border-transparent bg-transparent"
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
                    onmouseenter: move |_| on_preview_position.call(position),
                    onmouseleave: move |_| on_clear_preview.call(position),
                    onfocus: move |_| on_preview_position.call(position),
                    onblur: move |_| on_clear_preview.call(position),
                    if is_previewed {
                        span {
                            class: "pointer-events-none absolute inset-0 flex items-center justify-center bg-[radial-gradient(circle,rgba(253,230,138,0.38),rgba(245,158,11,0.18))] font-black uppercase tracking-[0.16em] text-amber-50 drop-shadow",
                            aria_hidden: "true",
                            "Target"
                        }
                    }
                }
            }
        }
        BoardTileSnapshot::Blocked => rsx! {
            div {
                role: "img", aria_label: "Row {row}, column {column}: Blocked",
                class: "relative aspect-[84/102] overflow-hidden rounded-lg border border-amber-700/70 bg-[repeating-linear-gradient(135deg,rgba(120,53,15,0.45)_0,rgba(120,53,15,0.45)_8px,rgba(30,41,59,0.65)_8px,rgba(30,41,59,0.65)_16px)] p-2 text-amber-200",
                span { class: "flex h-full items-center justify-center text-2xl", "╳" }
            }
        },
        BoardTileSnapshot::Occupied { controller, card } => {
            let owner = match controller {
                BoardSide::Blue => CardOwner::Human,
                BoardSide::Red => CardOwner::Cpu,
            };
            let preview_kind = preview_targets
                .iter()
                .find(|target| target.card_id == card.id)
                .map(|target| target.kind);
            let combat_power = combat.and_then(|combat| {
                if combat.attacker_id == card.id {
                    Some(("Attack", combat.attack_power))
                } else if combat.defender_id == card.id {
                    Some(("Defense", combat.defense_power))
                } else {
                    None
                }
            });
            rsx! { div { class: "aspect-[84/102]", aria_label: "Row {row}, column {column}", CardView { card, size: CardSize::Board, owner, selected: false, preview_kind, combat_power } } }
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
                                class: "aspect-[84/102] min-w-0 rounded-xl focus:outline-none focus:ring-4 focus:ring-amber-200/80 disabled:cursor-not-allowed disabled:opacity-50",
                                disabled: interaction != InteractionState::AwaitingPlayerAction,
                                aria_pressed: selected,
                                aria_label: accessible_name,
                                onclick: move |_| on_select_card.call(card_id),
                                CardView { card, size: CardSize::Hand, owner: CardOwner::Human, selected, preview_kind: None, combat_power: None }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CardView(
    card: CardSnapshot,
    size: CardSize,
    owner: CardOwner,
    selected: bool,
    preview_kind: Option<PreviewKind>,
    combat_power: Option<(&'static str, u8)>,
) -> Element {
    let custom_base_url = CUSTOM_CARD_ARTWORK.to_string();
    let fallback_url = FALLBACK_CARD_ARTWORK.to_string();
    let artwork = ArtworkResolver::new(&custom_base_url, &fallback_url)
        .resolve(card.definition_index)
        .expect("card snapshots must reference a catalog definition");
    let mut artwork_state = use_signal(|| ArtworkLoadState::Primary);
    let artwork_source = artwork_state().source(&artwork).to_owned();
    let owner_class = match owner {
        CardOwner::Human => "border-blue-300 bg-gradient-to-b from-blue-700 to-blue-950",
        CardOwner::Cpu => "border-rose-300 bg-gradient-to-b from-rose-700 to-rose-950",
    };
    let (owner_label, stats_mask_class) = match owner {
        CardOwner::Human => ("Player", "bg-blue-950/80"),
        CardOwner::Cpu => ("CPU", "bg-rose-950/80"),
    };
    let selected_class = if selected {
        "-translate-y-2 ring-4 ring-amber-200 shadow-amber-300/30"
    } else {
        "ring-1 ring-black/30"
    };
    let preview_class = match preview_kind {
        Some(PreviewKind::Attack) => {
            "outline outline-4 outline-dashed outline-rose-300 shadow-rose-400/60"
        }
        Some(PreviewKind::Capture) => {
            "outline outline-4 outline-double outline-cyan-200 shadow-cyan-300/60"
        }
        None => "",
    };
    let name_size = match size {
        CardSize::Board => "text-[10px]",
        CardSize::Hand => "text-xs",
    };
    let class = format!(
        "relative h-full w-full overflow-visible rounded-lg border-2 {owner_class} {selected_class} {preview_class} text-left shadow-lg transition"
    );
    let stats = format_card_stats(&card);
    let stats_label = format!(
        "Attack {}, class {}, physical defense {}, magic defense {}",
        card.stats.attack >> 4,
        card.stats.battle_class,
        card.stats.phys_defense >> 4,
        card.stats.mag_defense >> 4
    );
    rsx! {
        article {
            class,
            span { class: "sr-only", "Controlled by {owner_label}" }
            if let Some(preview_kind) = preview_kind {
                span {
                    class: "sr-only",
                    match preview_kind {
                        PreviewKind::Attack => "Will be attacked by this placement",
                        PreviewKind::Capture => "Will be captured directly by this placement",
                    }
                }
                div {
                    class: match preview_kind {
                        PreviewKind::Attack => "pointer-events-none absolute inset-x-1 top-1 z-30 rounded border border-rose-100 bg-rose-950/90 px-1 py-0.5 text-center text-[9px] font-black uppercase tracking-wider text-rose-100",
                        PreviewKind::Capture => "pointer-events-none absolute inset-x-1 top-1 z-30 rounded border border-cyan-100 bg-cyan-950/90 px-1 py-0.5 text-center text-[9px] font-black uppercase tracking-wider text-cyan-100",
                    },
                    aria_hidden: "true",
                    match preview_kind {
                        PreviewKind::Attack => "Attack",
                        PreviewKind::Capture => "Capture",
                    }
                }
            }
            img {
                src: artwork_source,
                alt: "",
                width: "84",
                height: "102",
                class: "absolute inset-0 h-full w-full rounded-md object-cover",
                onerror: move |_| artwork_state.write().use_fallback(),
            }
            div {
                class: "pointer-events-none absolute inset-0 z-20",
                for arrow in arrow_markers(card.arrows) { span { class: "absolute flex h-4 w-4 items-center justify-center text-sm font-black leading-none text-yellow-300 drop-shadow-[0_1px_1px_rgba(0,0,0,0.95)] {arrow.class}", aria_hidden: "true", "▲" } }
            }
            if let Some((role, power)) = combat_power {
                div {
                    class: "absolute left-1/2 top-1/2 z-30 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-yellow-200 bg-slate-950/90 px-2 py-1 font-mono text-sm font-black text-yellow-100 shadow-lg",
                    aria_label: "{role} power {power}",
                    "{power}"
                }
            }
            div {
                class: "absolute inset-x-1 bottom-2 z-10 overflow-hidden rounded text-center shadow backdrop-blur-[1px] {stats_mask_class}",
                h3 { class: "{name_size} truncate px-1 pt-0.5 font-bold leading-tight text-white", title: "{card.name}", "{card.name}" }
                div { class: "px-1 py-0.5 font-mono text-[11px] font-black tracking-[0.16em] text-amber-100", aria_label: stats_label, "{stats}" }
            }
        }
    }
}

struct ArrowMarker {
    class: &'static str,
}

fn arrow_markers(mask: u8) -> Vec<ArrowMarker> {
    const ARROWS: [&str; 8] = [
        "left-1/2 top-0 -translate-x-1/2 -translate-y-1/2",
        "right-0 top-0 translate-x-1/2 -translate-y-1/2 rotate-45",
        "right-0 top-1/2 translate-x-1/2 -translate-y-1/2 rotate-90",
        "bottom-0 right-0 translate-x-1/2 translate-y-1/2 rotate-[135deg]",
        "bottom-0 left-1/2 -translate-x-1/2 translate-y-1/2 rotate-180",
        "bottom-0 left-0 -translate-x-1/2 translate-y-1/2 -rotate-[135deg]",
        "left-0 top-1/2 -translate-x-1/2 -translate-y-1/2 -rotate-90",
        "left-0 top-0 -translate-x-1/2 -translate-y-1/2 -rotate-45",
    ];
    ARROWS
        .iter()
        .enumerate()
        .filter(|(index, _)| mask & (1 << index) != 0)
        .map(|(_, class)| ArrowMarker { class })
        .collect()
}

fn format_card_stats(card: &CardSnapshot) -> String {
    format!(
        "{:X}{}{:X}{:X}",
        card.stats.attack >> 4,
        card.stats.battle_class,
        card.stats.phys_defense >> 4,
        card.stats.mag_defense >> 4
    )
}

#[component]
fn HelpDialog(on_close: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { role: "dialog", aria_modal: "true", aria_labelledby: "help-title", class: "fixed inset-0 z-50 flex items-center justify-center bg-slate-950/75 p-6 backdrop-blur-sm",
            div { class: "w-full max-w-lg rounded-2xl border border-amber-200/50 bg-slate-900 p-8 shadow-2xl shadow-black",
                div { class: "flex items-start justify-between gap-4",
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.3em] text-amber-300", "Game guide" }
                        h2 { id: "help-title", class: "mt-2 font-serif text-3xl font-black text-amber-100", "How to play" }
                    }
                    button { autofocus: true, r#type: "button", aria_label: "Close help", class: "flex h-9 w-9 items-center justify-center rounded-full border border-slate-500 text-xl font-bold text-slate-100 hover:border-slate-200 focus:outline-none focus:ring-2 focus:ring-amber-200", onclick: move |event| on_close.call(event), "×" }
                }
                ol { class: "mt-6 space-y-3 text-sm leading-relaxed text-slate-200",
                    li { class: "flex gap-3", span { class: "font-bold text-amber-300", "1" } "Choose a card from your hand." }
                    li { class: "flex gap-3", span { class: "font-bold text-amber-300", "2" } "Play it on a marked open cell." }
                    li { class: "flex gap-3", span { class: "font-bold text-amber-300", "3" } "Yellow triangles show the directions a card can interact." }
                }
                div { class: "mt-6 rounded-lg border border-white/10 bg-slate-950/60 p-4 text-sm text-slate-300",
                    p { class: "font-bold uppercase tracking-widest text-amber-200", "Card stats" }
                    p { class: "mt-2", "Attack · Class · Physical defense · Magic defense" }
                    p { class: "mt-1 text-xs text-slate-400", "P Physical · M Magic · X Flexible · A Assault" }
                }
            }
        }
    }
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
    use crate::{models::core::card::BattleClass, test_support::card};

    #[test]
    fn arrow_markers_follow_all_eight_mask_bits() {
        assert!(arrow_markers(0).is_empty());
        assert!(arrow_markers(0b0000_0001)[0].class.contains("top-0"));
        assert!(arrow_markers(0b1000_0000)[0].class.contains("-rotate-45"));
        assert_eq!(arrow_markers(u8::MAX).len(), 8);
    }

    #[test]
    fn displayed_stats_match_the_tui_high_nibble_format() {
        let card = card(1, 0, BattleClass::Flexible, 0xaf, 0x31, 0x09);
        let snapshot = CardSnapshot::from(&card);

        assert_eq!(format_card_stats(&snapshot), "AX30");
    }
}
