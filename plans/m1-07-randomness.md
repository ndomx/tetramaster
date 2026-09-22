# M1.07 - Centralize Randomness

## Objective

Make every random outcome reproducible without exposing a concrete RNG through the
frontend contract.

## Prerequisites

- M1.06 is complete.
- The human has resolved whether card-stat generation preserves the characterized
  ranges or adopts corrected rules.

## Scope

- Remove global random calls from card generation and combat.
- Give session creation an explicit seed and own the resulting RNG state inside
  the engine boundary.
- Pass randomness explicitly to lower-level operations that need it.
- Use the same random source for board generation, hands, combat, and later AI
  selection.
- Add deterministic tests that report their seed on failure.
- Have the current TUI obtain a seed using existing dependencies.

## Non-goals

- No new RNG crate, dependency change, Cargo edit, AI policy change, frontend
  contract, or browser entropy implementation.

## Acceptance Criteria

- The same seed and action sequence produce the same board, hands, outcomes, and
  turn sequence.
- Domain value objects do not call ambient random functions.
- No borrowed `ThreadRng` lifetime remains on the session.
- Zero-valued and upper-bound stat behavior is explicitly tested.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```
