# M1.02 - Characterize Current Behavior

## Objective

Protect existing behavior before moving responsibilities. These tests describe
what the program does today; they do not silently declare suspected bugs correct.

## Prerequisites

- M1.01 is complete.
- The human has agreed on test placement and any fixture helpers.

## Kickoff Questions

- Identify current behaviors that look accidental and ask whether to characterize
  them, document them, or exclude them pending a later decision.
- Confirm representative seeds and board scenarios to retain for regression tests.

## Scope

- Add focused tests for board bounds, occupancy, placement, score calculation,
  hand removal, and game completion.
- Cover direct capture, opposed arrows, combat victory and defeat, combo
  propagation, effect ordering, and turn changes where current seams permit.
- Use guaranteed outcomes or recorded seeds for randomized behavior; never add
  flaky statistical assertions.
- Record current tie, random range, combo order, and runtime-ID behavior without
  resolving the deferred design questions in `00-decisions.md`.

## Non-goals

- No production refactor except a minimal behavior-neutral test seam approved at
  kickoff. No bug fixes, renames, dependency changes, or Cargo edits.

## Acceptance Criteria

- Tests fail when a covered behavior is deliberately broken.
- Randomized tests reproduce from a known seed or avoid nondeterministic outcomes.
- Suspected bugs are named in test comments or the handoff rather than corrected.
- The TUI remains playable.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
