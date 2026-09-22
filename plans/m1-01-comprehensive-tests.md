# M1.01 - Lock Current Behavior With Comprehensive Tests

## Objective

Build a broad regression suite around the working game before any structural
refactor begins. The suite must make unintended rule, state-machine, ownership,
and board changes visible to every later agent.

## Prerequisites

- Stage 0 is complete.
- No structural refactor has started.
- The human has reviewed the behavior matrix and identified known bugs or rules
  that should be documented rather than enshrined as intended behavior.

## Kickoff Questions

- Which current behaviors are known bugs, incomplete features, or deliberate
  deviations from Final Fantasy IX?
- Which representative board arrangements, battles, and combo chains does the
  human consider essential regression scenarios?
- Should tests use only module-local access, or may narrowly scoped test-only
  helpers be added after human approval?
- Is an automated complete-game smoke test sufficient for the current random
  session, with exact seeded sequences deferred until M1.07?

## Required Behavior Matrix

Cover at least:

- embedded catalog loading and every record's required current fields;
- position bounds, relative movement, and all eight directions;
- card arrow detection and displayed stat encoding;
- board construction invariants, row access, empty/blocked/occupied lookup,
  placement rejection, ownership changes, empty counts, and scores;
- hand lookup/removal and card identity behavior;
- physical, magical, flexible, and assault combat stat selection;
- guaranteed combat victory and defeat scenarios despite current randomness;
- no-arrow interactions, direct capture, opposed-arrow combat, multiple neighbors,
  effect priority, victory propagation, defeat behavior, and combo ordering;
- game start, active-player changes, player and CPU turns, pending-effect
  progression, end-turn behavior, terminal conditions, final scores, and current
  tie handling;
- at least one automated full-game invariant test that reaches completion through
  the current public behavior without terminal input.

For random behavior that cannot yet be seeded, assert stable invariants or use
inputs whose outcomes are guaranteed. Do not add probabilistic assertions that
can occasionally fail. Record exact-sequence gaps for M1.07.

## Scope

- Add unit and behavior tests to the existing binary crate and modules.
- Add reusable test fixtures/builders under test-only configuration when they
  improve clarity.
- Make only minimal, human-approved production changes required to observe
  behavior; do not redesign APIs for testing.
- Document suspicious behavior in test names, comments, or the handoff without
  silently correcting it.

## Non-goals

- No `src/lib.rs`, module moves, renames, rule fixes, RNG redesign, dependency
  changes, Cargo edits, or Dioxus work.

## Acceptance Criteria

- Every row of the agreed behavior matrix has direct test coverage or a documented
  reason it must wait for a named later step.
- Tests exercise outcomes and invariants, not merely constructors or getters.
- Randomized tests are non-flaky and clearly identify deferred seeded coverage.
- A deliberate local mutation to each major covered subsystem causes a relevant
  test to fail; such mutations are reverted before completion.
- The test suite passes repeatedly, and the TUI remains playable.
- The handoff lists known untested risks and the later step responsible for each.

## Verification

```sh
cargo fmt --check
cargo test
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```

Running the test suite twice is intentional while ambient randomness still exists.

## Completion Record

Completed with 28 tests covering the agreed behavior matrix. Test-only fixtures
provide deterministic cards and boards, and a test-only combat roll hook covers
guaranteed victory, defeat, tie, and battle-class defense selection without
changing the production RNG API.

Characterized suspicious current behavior:

- `Board::place_card` overwrites blocked and occupied tiles; placement rejection
  currently belongs to the `Game` layer.
- Combat ties are defeats.
- A defeated challenger changes ownership and immediately ends effect processing.

Deferred risks:

- Exact generated-stat, starting-player, CPU-choice, and combat-roll sequences
  remain untested until M1.07 centralizes injectable randomness.
- Final-score tie handling remains inline in the interactive binary and treats a
  tie as a CPU win; its public result representation is deferred to M1.09.
