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
- Keep terminal rendering and input separate from game decisions.

## Non-goals

- Rewriting the game from scratch.
- Changing game balance or rules during structural work.
- Adding a GUI, networking, persistence, or a large framework.
- Generalizing the engine for unrelated card games.

## Target Responsibilities

The intended dependency direction is:

```text
main -> game session -> rules -> domain
  |          |            |
  +--------> UI <----- outcomes/events

assets -> domain
AI ----> legal game actions
```

- `domain`: game data and small invariants, such as cards, positions, players,
  placed cards, and board storage.
- `rules`: pure operations for placement, combat, captures, and combos. These
  functions receive all inputs explicitly and do not print or read terminal input.
- `game`: match/session orchestration, turn progression, active player, phase,
  scores, and applying rule outcomes.
- `ai`: CPU move selection only. It chooses from legal actions but does not apply
  them.
- `ui`: terminal input and ASCII presentation. It translates user input into game
  actions and renders state and events.
- `assets`: loading and validation of the embedded, read-only card catalog.

A likely end-state layout is:

```text
src/
  main.rs
  lib.rs
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
    session.rs
    state.rs
  ai/
    mod.rs
    random.rs
  assets/
    catalog.rs
  ui/
```

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

## Work Plan

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
| `Action` | `PlayCardAction` | Distinguishes a player move from internal effects. |
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
- [ ] Route player and CPU moves through the same validated action API.
- [ ] Have the session apply rule outcomes and advance the turn explicitly.
- [ ] Model illegal moves with a typed error such as `GameError` rather than
  `Result<T, String>`.
- [ ] Represent observable results as values such as `CombatOutcome` and
  `GameEvent` so the UI can render them.

The session should answer “what happens next?” while the rule modules answer “is
this move legal?” and “what does this interaction produce?”

### Step 5: Centralize randomness

- [ ] Remove global random calls from card generation and combat.
- [ ] Pass an RNG into operations that need it, or give the session one owned RNG.
- [ ] Pass the same randomness boundary to CPU move selection.
- [ ] Use seeded RNGs in tests so failures are reproducible.
- [ ] Verify zero-valued stat ranges and whether generated ranges should include
  their upper bound.

Randomness is an input to the rules, not hidden ambient state. The concrete RNG
type does not need to leak through every model type.

### Step 6: Isolate AI and presentation

- [ ] Move CPU move selection out of the session and into `ai`.
- [ ] Give the AI a read-only game view plus a set of legal actions.
- [ ] Remove all `println!` calls from domain and rule modules.
- [ ] Let the terminal layer render returned events and outcomes.
- [ ] Consolidate duplicated card-line rendering shared by hand and board views.
- [ ] Simplify UI lifetimes after the session no longer stores a borrowed RNG.

### Step 7: Final cleanup

- [ ] Review visibility and keep internal types and methods private where possible.
- [ ] Remove empty utility abstractions and relocate helpers beside their callers.
- [ ] Run `cargo fmt`, `cargo test`, and
  `cargo clippy --all-targets -- -D warnings`.
- [ ] Update `README.md` and `AGENTS.md` to match the final layout.
- [ ] Record any intentional rule deviations from Final Fantasy IX.

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

## Completion Criteria

The refactor is complete when:

- game rules can be exercised through tests without terminal I/O;
- all random behavior can be reproduced with a seed;
- the session coordinates rules but does not implement combat, capture, or AI;
- UI modules do not decide game outcomes;
- illegal states and moves have explicit representations;
- names consistently reflect the game's domain; and
- the full format, test, and lint checks pass.
