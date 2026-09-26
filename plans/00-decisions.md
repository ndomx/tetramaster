# Established Decisions And Deferred Rules

Read this file before every implementation step. Established decisions remain in
force until the human explicitly changes them.

## Project And Milestone Boundaries

- Use one Cargo package with a shared library and application binaries.
- Comprehensive behavior-locking tests are the first Milestone 1 deliverable and
  must pass before module extraction, renaming, or other structural refactoring.
- Milestone 1 must not edit `Cargo.toml` or change dependencies.
- Milestone 1 retains `src/main.rs` as the TUI entry point.
- Dioxus, RON, artwork support, platform features, and binary separation begin in
  Milestone 2 because they require manifest or dependency changes.
- The Dioxus application is browser-only and client-rendered. Mobile, Fullstack,
  SSR, networking, and persistence are out of scope.

## Catalog

- The existing CSV contains the complete canonical card catalog today, but CSV is
  not the catalog's ultimate storage format.
- During Milestone 1, the embedded CSV remains read-only and behaviorally
  unchanged.
- At the start of Milestone 2, migrate the catalog to embedded, human-editable RON
  with named fields and typed battle classes.
- RON is preferred because definitions will gain artwork and may gain optional,
  list, or nested fields that do not scale cleanly as CSV columns.
- `CardDefinition` is canonical catalog data; `Card` is a randomized playable
  instance referencing a `&'static CardDefinition`.
- Parse the embedded catalog once into an immutable global collection. Runtime
  editing or replacement is out of scope.

### M2.01 Catalog And Artwork Decisions

- Runtime `CardDefinition::index` remains zero-based and is assigned by canonical
  RON catalog order. Artwork numbering is separately one-based and zero-padded
  from `001` through `100`.
- Card artwork is 84x102 PNG. Copyrighted artwork is not stored in the repository;
  each definition records its corresponding Fandom page reference using the
  `CardNNN.png` query parameter.
- The repository contains one original 84x102 PNG fallback for unavailable remote
  artwork. Artwork metadata includes the source URL, fallback path, and dimensions;
  decoded image data remains outside the core model.
- RON fields are required. Stats use their expanded `u8` gameplay values and
  battle classes use named typed variants. Catalog validation enforces unique
  names and artwork URLs, expected stat tiers, exact artwork dimensions, and the
  position-derived artwork URL.

## Vocabulary

- `Card`: a playable card instance, including while held in a hand.
- `BoardCard`: a card on the board together with its current controller.
- `Tile`: `Empty`, `Blocked`, or `Occupied(BoardCard)`.
- `GameAction::PlayCard`: a request to move a card from hand to board.
- `GameEvent::CardPlaced`: the observable placement result.
- Prefer `CardTileView`, `BlockedTileView`, and `EmptyTileView` for TUI rendering
  names. Dioxus may represent all tile states with `BoardCell`.

## Model Module Layout

- `models` remains the umbrella for game data rather than introducing a separate
  top-level `domain` module.
- `models::core` contains every essential Tetra Master concept and owns local
  invariants. It is grouped into `board`, `card`, and `geometry`; `Player` stays
  directly under `core` because it does not naturally belong to those groups.
- `models::session` contains application-level state and workflow for a running
  match, including actions, active-player state, effects, phases, pending work,
  and `GameSession`.
- The dependency direction is `models::session -> models::core`. Core models
  must not depend on session coordination, commands/rules, AI, or UI code.
- Important types are re-exported from their conceptual group, producing paths
  such as `models::core::board::Board`, `models::core::card::Card`,
  `models::core::geometry::Position`, and `models::session::GameSession`.
- Assets own catalog record parsing and construct `CardDefinition`; core card
  definitions do not depend on the assets layer.
- Do not add speculative model submodules. Frontend contract types introduced in
  M1.09 stay with `models::session` unless their actual size and responsibilities
  justify a separately approved grouping.

### M1.03 Transitional Naming Decisions

- Rename the existing placement-request struct from `Action` to `GameAction`
  during M1.03, but do not turn it into the planned frontend-contract enum yet.
  M1.09 must still settle and implement the public `GameAction::PlayCard` shape;
  the M1.03 name alone does not make the current struct that contract.
- Use the singular `available_position` for the current board method because it
  randomly returns one `Option<Position>`. If a later rules or AI API exposes all
  legal positions, that separate collection API may use a plural name.
- M1.05 replaces that board-level random selection with
  `rules::placement::legal_positions`; session coordination temporarily chooses
  a CPU target from the returned positions until AI extraction in M1.08.

## Frontend Contract

- The TUI and Dioxus app must drive the engine through the same public contract.
- `GameAction` represents requests. Initially it contains
  `PlayCard { card_id, position }`.
- `GameEvent` represents observable facts, not internal pending effects.
- `InteractionState` is `AwaitingPlayerAction`, `Advancing`, or `Finished`.
- `GameSnapshot` is an owned, inexpensive `Clone + PartialEq` projection. It shows
  the board, human hand, CPU hand count, ownership, scores, phase, result, and
  currently legal human actions without exposing mutable internals or hidden CPU
  cards.
- `GameUpdate` contains the ordered events from one player-visible transition,
  the snapshot after those events, and the resulting interaction state.
- `dispatch` accepts frontend actions only during `AwaitingPlayerAction`.
- `advance` performs exactly one automatic player-visible transition. Purely
  internal bookkeeping may be collapsed until an event or stable interaction
  state is reached.
- A successful `dispatch` or `advance` returns one `GameUpdate`; repeated
  `advance` calls resolve remaining CPU, combat, capture, combo, and turn events.
- A failed operation returns `GameError` and leaves the session unchanged.
- Frontends never submit CPU actions. During `advance`, the session asks the AI to
  choose a `GameAction`, then applies it through the same internal validation path
  used for human actions.
- Timing, selection, hover, animation progress, formatting, and I/O belong to the
  frontend adapters.

### M2.02 Web Bootstrap Decisions

- Keep one Cargo package with the shared library and two binaries named
  `tetramaster-tui` and `tetramaster-web`.
- Use Dioxus 0.7.10 with `tui` and `web` package features. `tui` remains the
  default feature for native development.
- Crossterm is an optional native-only dependency. Terminal modules are also
  source-gated away from WebAssembly builds.
- The web target is client-rendered WebAssembly and uses the Dioxus-managed
  Tailwind integration. `tailwind.css` is the source and
  `assets/tailwind.css` is the generated stylesheet.
- Router, Fullstack, SSR, backend, desktop, and mobile features remain out of
  scope. The bootstrap is handwritten instead of generated with `dx new`.
- WebAssembly enables the `getrandom` browser backend so the shared seeded RNG
  dependency compiles for `wasm32-unknown-unknown`.

Candidate event families are game start, turn start, card placement, combat
resolution, ownership change, turn end, and game finish. Exact variant payloads
must be agreed with the human during M1.09 before implementation.

### M1.09 Frontend Contract Decisions

- A tied final score is represented explicitly as `GameResult::Draw`.
- Runtime card IDs remain `u64` random values and are guaranteed unique within a
  session by regenerating a card if its ID collides during hand construction.
- Each successful `dispatch` or `advance` returns exactly one observable event.
  Internal bookkeeping is collapsed until that event or a stable interaction
  state is reached.
- The initial event variants are game started, turn started, card placed, combat
  resolved, ownership changed, turn ended, and game finished. Combat resolution
  and its resulting ownership change are separate transitions so snapshots show
  state after exactly the reported event.
- M1.09 initially exposed board ownership through `PlayerSide::{Human, Cpu}`
  rather than internal player IDs. M1.11 supersedes that representation with
  `BoardSide`; snapshots still include visible board and card data, the human
  hand, CPU hand count, scores, phase, result, and legal human actions while CPU
  cards remain hidden.

### M1.11 Board Control Decisions

- `BoardSide::{Blue, Red}` represents match-local control of cards on the board.
  It is independent of player identity and starting-turn order.
- For the current human-versus-CPU session, the player controls Blue and the CPU
  controls Red. How participants choose or receive colors in a future PvP mode is
  deferred until that mode is designed.
- Each `Player` carries its assigned `BoardSide`. Session orchestration reads the
  human and CPU assignments from those players when placing, capturing, and
  scoring cards rather than hardcoding colors at those call sites.
- Starting-player selection remains random; Blue does not always start.
- Board snapshots expose `BoardSide` directly. Capture events use
  `GameEvent::ControlChanged` and `ControlChangeReason` rather than ownership
  terminology.
- `Player` has no numeric ID. M1.12 removes the obsolete random draws that once
  generated those IDs; historical seeds therefore produce new deterministic
  sequences.

### M1.12 Cleanup Decisions

- `GameSession::{new, snapshot, dispatch, advance, interaction_state}` and the
  frontend contract types are the supported session interface. The duplicate
  pre-contract execution path has been removed, and behavioral tests drive the
  same contract as frontends.
- Session state, phases, pending effects, and active-player bookkeeping are
  private implementation details rather than a second mutable public API.
- CPU move selection receives legal actions and seeded randomness only; unused
  board and hand inputs are not part of its interface.
- Catalog records and ASCII rendering modules remain internal to their adapters.
- Active-player state uses `Option<PlayerSide>` rather than a duplicate internal
  enum, and player construction receives its `BoardSide` explicitly.
- Board controller mutation returns `BoardControlError`; string errors are
  reserved for the public session boundary's internal-failure reporting.
- `CaptureOutcome` reports only the previous controller. `CaptureKind` remains in
  the capture request as explicit rule context.
- Presentation formatting does not live on core `Card`; the terminal adapter
  formats snapshot data instead.
- The top-level engine, rules, AI, asset, UI, and utility modules remain public
  for now, while incidental implementation modules and session bookkeeping stay
  private.
- Public API compatibility with pre-Milestone-2 refactor shapes is a non-goal.
  Removing obsolete aliases, methods, fields, and payloads is preferred over
  retaining compatibility shims.

## Deferred Game Rules

Structural steps preserve and characterize current behavior. Resolve each item
with the human before the listed implementation step changes or formalizes it.

| Question | Preserve until | Must resolve before |
| --- | --- | --- |
| Is a tied final score a draw or a CPU win? | Existing winner behavior | M1.09 final result API |
| Should generated stat ranges preserve their current distribution? | Characterization tests | M1.07 RNG implementation |
| What is the exact combo traversal and ordering? | Characterized behavior retained | Resolved for M1.06; see below |
| Does every playable card require a unique runtime ID? | Existing random ID behavior | M1.09 public action/event IDs |

M1.01 records current behavior without treating it as the final design decision.

### M1.06 Combat And Combo Decisions

- M1.06 preserves the characterized deterministic ordering for multiple battles.
  This is an internal ordering choice, not a final frontend contract; a later
  rules/API step may accept an injected order, including a randomized order.
- M1.06 preserves one-hop combo discovery and queues each combo capture as an
  individual pending effect. The original game presents all captures from one
  combo simultaneously; a later event or animation design may group them without
  silently changing the resulting ownership transitions.

### M1.07 Randomness Decisions

- A per-session `GameRng` wraps the existing dependency's seeded `StdRng` and is
  owned by `GameSession`. There is no process-global random state, borrowed
  `ThreadRng`, or concrete generator exposed through the frontend-facing API.
- The TUI creates a fresh seed, constructs `GameRng`, and moves it into the
  session. The seed is not printed as standalone debug output. A recorded seed
  and identical action sequence reproduce the complete game in deterministic
  tests.
- Generated card stats use inclusive `0..=base` ranges. A zero base produces
  zero, and the base value itself is reachable.
- The random combat bonus uses saturating addition, so power is capped at
  `u8::MAX` instead of overflowing when a generated stat is near its upper bound.
