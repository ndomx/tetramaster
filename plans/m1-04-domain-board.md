# M1.04 - Separate Domain And Board Responsibilities

## Objective

Move data-centric types out of `models` and make board invariants explicit without
moving combat, capture, AI, session, or presentation behavior with them.

## Prerequisites

- M1.03 is complete.
- The human has approved the exact target module tree for the types found in the
  current repository.

## Scope

- Create `domain` modules for cards, definitions, stats, battle class, directions,
  positions, players, tiles, `BoardCard`, and board storage.
- Keep `GamePhase`, pending work, and session coordination outside `domain`.
- Give the board one checked placement operation that rejects out-of-bounds,
  blocked, and occupied positions without mutation.
- Keep local queries and invariants near the data they govern.
- Keep catalog loading under `assets`; retain the embedded CSV during Milestone 1.

## Non-goals

- No RON migration, stat-generation changes, rule extraction, frontend contract,
  dependency changes, or Cargo edits.

## Acceptance Criteria

- Domain modules do not depend on TUI, session, AI, or command/rule orchestration.
- Failed placement is atomic and covered by tests.
- Existing behavior tests and the TUI remain green.
- `AGENTS.md` is updated in this same step to describe the new module layout.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```
