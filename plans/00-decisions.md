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

## Vocabulary

- `Card`: a playable card instance, including while held in a hand.
- `BoardCard`: a card on the board together with its current owner.
- `Tile`: `Empty`, `Blocked`, or `Occupied(BoardCard)`.
- `GameAction::PlayCard`: a request to move a card from hand to board.
- `GameEvent::CardPlaced`: the observable placement result.
- Prefer `CardTileView`, `BlockedTileView`, and `EmptyTileView` for TUI rendering
  names. Dioxus may represent all tile states with `BoardCell`.

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

Candidate event families are game start, turn start, card placement, combat
resolution, ownership change, turn end, and game finish. Exact variant payloads
must be agreed with the human during M1.09 before implementation.

## Deferred Game Rules

Structural steps preserve and characterize current behavior. Resolve each item
with the human before the listed implementation step changes or formalizes it.

| Question | Preserve until | Must resolve before |
| --- | --- | --- |
| Is a tied final score a draw or a CPU win? | Existing winner behavior | M1.09 final result API |
| Should generated stat ranges preserve their current distribution? | Characterization tests | M1.07 RNG implementation |
| What is the exact combo traversal and ordering? | Characterization tests | M1.06 rule extraction |
| Does every playable card require a unique runtime ID? | Existing random ID behavior | M1.09 public action/event IDs |

M1.01 records current behavior without treating it as the final design decision.
