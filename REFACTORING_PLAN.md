# Refactoring Plan

This document tracks the planned cleanup of the Tetra Master implementation. The
work should be incremental: establish the current behavior with tests, move one
responsibility at a time, and keep the game playable after every step.

## Goals

- Make the game rules readable and testable without a terminal.
- Give each module one clear responsibility.
- Make randomness deterministic in tests.
- Use consistent Tetra Master vocabulary throughout the codebase.
- Remove unused APIs and accidental complexity.
- Keep all frontend rendering and input separate from game decisions.
- Give the terminal and future web frontends the same action, event, state, and
  error contract.
- Keep the shared game library compatible with the browser's WebAssembly target.

## Non-goals

- Rewriting the game from scratch.
- Changing game balance or rules during structural work.
- Adding Dioxus or writing RSX components during the current refactor milestone.
- Adding a backend, SSR, networking, persistence, or mobile application.
- Generalizing the engine for unrelated card games.

## Target Responsibilities

The intended dependency direction is:

```text
TUI adapter ---------------\
                            +-> frontend contract -> game session -> rules -> domain
future Dioxus web adapter -/                            |
                                                         +-> AI

assets -> domain
```

- `domain`: game data and small invariants, such as cards, positions, players,
  placed cards, and board storage.
- `rules`: pure operations for placement, combat, captures, and combos. These
  functions receive all inputs explicitly and do not print or read terminal input.
- `game`: match/session orchestration, turn progression, active player, phase,
  scores, applying rule outcomes, and the frontend-neutral integration contract.
- `ai`: CPU move selection only. It chooses from legal actions but does not apply
  them.
- `tui`: terminal input and ASCII presentation. It translates terminal input into
  shared game actions and renders shared snapshots and events.
- `web_ui`: Dioxus components added only in the later web milestone. They use the
  same contract as the TUI.
- `assets`: loading and validation of the embedded, read-only card catalog.

A likely end-state layout is:

```text
src/
  lib.rs
  bin/
    tui.rs
  domain/
    board.rs
    card.rs
    direction.rs
    player.rs
    position.rs
  rules/
    capture.rs
    combat.rs
    placement.rs
  game/
    action.rs
    event.rs
    interaction.rs
    session.rs
    snapshot.rs
    state.rs
  ai/
    mod.rs
    random.rs
  assets/
    catalog.rs
  tui/
```

The Dioxus milestone later adds `src/bin/web.rs` and `src/web_ui/`; it does not
replace or fork the game library.

This is a direction, not a requirement to create every module immediately. A
module should exist only when it has a distinct responsibility and enough code to
justify the boundary.

## Established Decisions

### Card catalog

`src/assets/card_records.csv` is the definitive and complete source of available
card definitions. It is application data, not user input or mutable runtime
state.

- Keep the CSV embedded in the executable with `include_str!`.
- Parse it once into an immutable global catalog using `LazyLock`.
- Do not add runtime loading, editing, or replacement of the catalog.
- Rename `CardAsset` to `CardDefinition`; it represents canonical game data, not
  a visual or audio asset.
- Keep the Serde input type private to the loader and name it `CsvCardRecord`.
- Let each playable `Card` hold a `&'static CardDefinition`. This keeps canonical
  names and base stats in one place and does not introduce a non-static lifetime.
- Force catalog initialization during application startup so invalid embedded
  data fails immediately rather than during hand generation.
- Validate the complete embedded catalog in a test, including row context and the
  exact four-character stats format.
- Expose catalog access through a read-only slice or iterator.

`CardDefinition` is the catalog entry, `Card` is a randomized playable instance,
and `CardStats` contains the instance's rolled combat values. The catalog's base
or limit stats may use the same value type, but fields should clearly distinguish
definition limits from runtime values.

### Frontend integration contract

The TUI is the first adapter for the same public contract that the Dioxus web app
will use later. There must not be separate terminal and web paths through the
game engine.

The contract consists of:

- `GameAction`: a request submitted to the session, initially including
  `PlayCard { card_id, position }`. The player and AI select from the same legal
  action representation.
- `GameEvent`: an immutable description of an observable result, such as a game
  or turn starting, a card being placed, combat resolving, ownership changing,
  or the game finishing. Internal pending effects are not UI events.
- `InteractionState`: tells an adapter whether the session is
  `AwaitingPlayerAction`, `Advancing`, or `Finished`.
- `GameSnapshot`: a read-only, frontend-neutral projection containing everything
  needed to render the board, the player's visible hand, the opponent's hand
  count, ownership, scores, phase, and legal choices. It must not reveal hidden
  information or expose mutable board or player internals.
- `GameError`: typed invalid-action and invalid-state failures that adapters can
  present without terminating.

`GameSession` should expose a small integration API along these lines:

```text
snapshot() -> GameSnapshot
interaction() -> InteractionState
legal_actions() -> collection of GameAction
dispatch(GameAction) -> Result<Vec<GameEvent>, GameError>
advance() -> Result<Vec<GameEvent>, GameError>
```

Both adapters follow the same sequence:

```text
render snapshot and newly emitted events
    |
    +-- AwaitingPlayerAction -> collect one action -> dispatch
    +-- Advancing ----------> optionally pace presentation -> advance
    +-- Finished -----------> render the result and stop
```

The TUI may block while collecting input and may use `thread::sleep` for pacing.
The web adapter will use event handlers and asynchronous timing. Those are adapter
details; neither belongs in `GameSession`. Adapters must not call rule functions,
mutate the board directly, choose CPU moves, or infer state transitions.

## Milestone 1: Engine Refactor and Frontend Preparation

### Step 0: Record the plan

- [x] Add this document.
- [ ] Review and amend the proposed module boundaries and vocabulary before code
  changes begin.

### Step 1: Characterize current behavior

- [ ] Add `src/lib.rs` so domain and rule code can be tested independently of the
  executable.
- [ ] Add focused tests for board bounds, occupied-tile rejection, card removal
  from a hand, and score calculation.
- [ ] Add a catalog test that parses every embedded CSV record and validates its
  name, exact stats format, battle class, and numeric values.
- [ ] Add combat tests for victory, defeat, and each battle-class stat pairing.
- [ ] Add effect tests for direct capture, opposed arrows, combo propagation, and
  effect ordering.
- [ ] Add session tests for turn changes and game completion.
- [ ] Decide and test how tied final scores are represented.

The tests in this step describe existing behavior. Any suspected game-rule bug
should be documented separately and changed only after the structural refactor.

### Step 2: Establish consistent vocabulary

- [ ] Rename types and methods in small, mechanical commits.
- [ ] Remove APIs proven unused after checking both production code and tests.
- [ ] Replace concrete collection references such as `&Vec<Card>` with slices
  where callers only need to read a sequence.
- [ ] Keep names based on game concepts rather than their current rendering or
  storage representation.

Proposed naming changes:

| Current | Proposed | Reason |
| --- | --- | --- |
| `CardAsset` | `CardDefinition` | It is canonical game data rather than a presentation asset. |
| `CardRecord` | `CsvCardRecord` | It is private deserialization plumbing for the embedded CSV. |
| `TileCard` | `PlacedCard` | Describes a card's role on the board. |
| `Action` | `GameAction` | Defines the command type shared by every frontend and the AI. |
| `EffectInstance` | `PendingEffect` | Describes an effect waiting to be resolved. |
| `Effect::Capture` | `Effect::DirectCapture` | Separates uncontested capture from combat victory. |
| `GameState` | `GamePhase` | The type represents a phase in the match lifecycle. |
| `find_available` | `available_positions` | Makes the returned collection and meaning explicit. |
| `find_placed_by_id` | `position_of_card` | Names the result rather than the search mechanism. |
| `facing` | `directions` or `arrows` | Avoids an ambiguous verb-like name. |

`ActivePlayer::None` should be revisited after session states are separated. If a
game phase guarantees whether a player is active, the impossible value should be
removed rather than renamed.

### Step 3: Separate domain data from rules

- [ ] Move data-centric types from `models` into `domain`.
- [ ] Keep only local invariants and basic queries on domain types.
- [ ] Keep catalog parsing under `assets`, while placing `CardDefinition`, `Card`,
  and `CardStats` with the domain types.
- [ ] Preserve `Card` references to immutable `&'static CardDefinition` values;
  remove the unused catalog index unless a concrete identity requirement appears.
- [ ] Make board placement validate bounds and occupancy through one explicit API,
  such as `try_place_card`.
- [ ] Move combat calculation from `commands/attack.rs` into `rules/combat.rs`.
- [ ] Move capture and combo discovery from `commands/effects.rs` into focused rule
  modules.
- [ ] Delete the `commands` module once its responsibilities have moved.

Rule functions should return outcomes describing what happened. They should not
mutate unrelated session state, choose CPU actions, print diagnostics, or render
messages.

### Step 4: Reduce `Game` to session orchestration

- [ ] Rename or replace `Game` with `GameSession` once its role is narrow enough.
- [ ] Keep ownership of the board, hands, phase, active player, and match history in
  the session layer.
- [ ] Route player and CPU moves through the same validated `GameAction` API.
- [ ] Have the session apply rule outcomes and advance the turn explicitly.
- [ ] Model illegal moves with a typed error such as `GameError` rather than
  `Result<T, String>`.
- [ ] Return ordered `GameEvent` values for every observable state change.
- [ ] Separate internal effect scheduling from events exposed to frontends.
- [ ] Expose legal actions so frontends and the AI do not recreate placement
  rules.

The session should answer “what happens next?” while the rule modules answer “is
this move legal?” and “what does this interaction produce?”

### Step 5: Centralize randomness

- [ ] Remove global random calls from card generation and combat.
- [ ] Give the session an owned, seedable RNG; do not retain a borrowed
  `ThreadRng` or expose a concrete RNG type in the frontend API.
- [ ] Pass the same randomness boundary to CPU move selection.
- [ ] Use seeded RNGs in tests so failures are reproducible.
- [ ] Let each executable obtain a platform-appropriate seed when starting a new
  session.
- [ ] Verify zero-valued stat ranges and whether generated ranges should include
  their upper bound.

Randomness is an input to the rules, not hidden ambient state. The concrete RNG
type does not need to leak through every model type.

### Step 6: Isolate AI and presentation

- [ ] Move CPU move selection out of the session and into `ai`.
- [ ] Give the AI a read-only game view plus a set of legal actions.
- [ ] Remove all `println!` calls from domain and rule modules.
- [ ] Keep colors, timing, formatted terminal lines, and I/O out of the shared
  library.
- [ ] Consolidate duplicated card-line rendering shared by hand and board views.
- [ ] Simplify UI lifetimes after the session no longer stores a borrowed RNG.

### Step 7: Establish the shared frontend contract

- [ ] Add `GameAction`, `GameEvent`, `InteractionState`, `GameSnapshot`, and
  `GameError` as frontend-neutral types.
- [ ] Implement `snapshot`, `interaction`, `legal_actions`, `dispatch`, and
  `advance` on `GameSession`.
- [ ] Ensure every successful dispatch or advance emits events in a deterministic
  order.
- [ ] Include ownership as a domain side/player concept in snapshots rather than
  requiring frontends to compare raw player IDs.
- [ ] Keep selection, hover state, animation progress, and presentation delays out
  of snapshots.
- [ ] Add integration tests that drive a seeded session entirely through the
  public contract until it finishes.
- [ ] Test invalid actions through the same contract and verify that they do not
  partially mutate the session.

### Step 8: Adapt and preserve the TUI

- [ ] Move the terminal entry point to `src/bin/tui.rs` and terminal presentation
  to `src/tui/`.
- [ ] Make Crossterm an optional, TUI-only dependency so it is not compiled into
  the browser target.
- [ ] Render only `GameSnapshot` and `GameEvent` values; do not read `GameSession`
  fields directly.
- [ ] Convert terminal input into a complete `GameAction` and submit it through
  `dispatch`.
- [ ] Use `InteractionState` to decide whether to prompt, call `advance`, or show
  the final result.
- [ ] Keep terminal blocking and optional sleeps inside the TUI adapter.
- [ ] Display `GameError` values and retry input instead of unwrapping or silently
  discarding errors.
- [ ] Keep the current ASCII presentation functional without requiring a broader
  TUI redesign.

At the end of this step, the TUI must demonstrate the exact action/event loop the
future Dioxus app will use. Only the mechanism for receiving input, scheduling
automatic advancement, and drawing output should differ.

### Step 9: Verify portability and finish the refactor

- [ ] Review visibility and keep internal types and methods private where possible.
- [ ] Remove empty utility abstractions and relocate helpers beside their callers.
- [ ] Verify the shared library with
  `cargo check --lib --target wasm32-unknown-unknown`.
- [ ] Run `cargo fmt`, `cargo test`, and
  `cargo clippy --all-targets -- -D warnings`.
- [ ] Update `README.md` and `AGENTS.md` to match the final layout.
- [ ] Record any intentional rule deviations from Final Fantasy IX.

## Milestone 2: Dioxus Web Frontend

This milestone starts only after Milestone 1 is complete. It consumes the shared
contract without adding web-specific branches to the engine.

### Step 10: Add the browser application

- [ ] Add Dioxus with its web feature and create `src/bin/web.rs`.
- [ ] Add `src/web_ui/` for RSX components, styles, and browser-only presentation
  state.
- [ ] Configure a client-rendered web build. Do not add Fullstack, SSR, a backend,
  mobile targets, or routing unless a later feature requires them.
- [ ] Keep Dioxus types, signals, callbacks, and asynchronous timers out of the
  shared game library.

### Step 11: Build the RSX component tree

- [ ] Implement `GameApp`, `Score`, `Board`, `BoardCell`, `Card`, `Hand`,
  `GameStatus`, and `GameOverDialog` components.
- [ ] Use one reusable card component for cards in the hand and on the board.
- [ ] Keep the board at a stable 4x4 layout and visibly distinguish blocked,
  empty, occupied, selected, legal, and contested cells.
- [ ] Communicate ownership with more than color alone.
- [ ] Support desktop browser widths and keyboard-accessible controls; mobile
  layouts are not a project target.

### Step 12: Connect Dioxus to the engine

- [ ] Store `GameSession` in Dioxus state without changing its public API.
- [ ] Keep selected card, hover state, modal state, and animation progress as web
  presentation state.
- [ ] Derive enabled cards and board cells from `legal_actions`.
- [ ] Submit clicks as `GameAction` values through `dispatch`.
- [ ] Render returned `GameEvent` values and asynchronously call `advance` while
  the interaction state is `Advancing`.
- [ ] Handle `GameError` without panicking or losing the current session.
- [ ] Add web-facing tests for selection and dispatch wiring, then verify a full
  playable match in a desktop browser.

### Step 13: Decide whether to retain the TUI

- [ ] Keep the TUI as an optional binary if its dependency and maintenance cost
  remain small.
- [ ] Confirm both frontends still consume the same public contract.
- [ ] Remove the TUI only as a deliberate follow-up decision, not as part of the
  web implementation.

## Decisions Needed

These should be resolved before the affected step, but they do not block the
initial characterization tests:

- Whether a tied final score is a draw or is intentionally awarded to the CPU.
- Whether card generation should preserve the current random distribution or be
  corrected to match known Tetra Master rules.
- Whether combo effects resolve depth-first, breadth-first, or in another defined
  order when multiple chains are possible.
- Whether each playable `Card` needs a unique runtime identifier; multiple cards
  may always reference the same `CardDefinition`.

## Milestone 1 Completion Criteria

The engine refactor and frontend preparation are complete when:

- game rules can be exercised through tests without terminal I/O;
- all random behavior can be reproduced with a seed;
- the session coordinates rules but does not implement combat, capture, or AI;
- the public frontend contract is the only way the TUI drives the session;
- the TUI remains fully playable using snapshots, actions, events, interaction
  state, and typed errors;
- automatic progression is driven through `advance` rather than polling or
  inferred by the adapter;
- frontend modules do not decide game outcomes or mutate game internals;
- the shared library compiles for `wasm32-unknown-unknown` without Crossterm;
- illegal states and moves have explicit representations;
- names consistently reflect the game's domain; and
- the full format, test, and lint checks pass.

Milestone 2 is complete when the Dioxus web app is fully playable through that
same contract, all web-only state remains in `web_ui`, and retaining or removing
the optional TUI has been decided explicitly.
