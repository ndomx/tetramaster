# M1.04 - Separate Core And Session Model Responsibilities

## Objective

Organize `models` around an essential core and application-level session state,
and make board invariants explicit without moving combat, capture, AI, or
presentation behavior with them.

## Prerequisites

- M1.03 is complete.
- The human has approved `models::{core, session}` and the nested `board`, `card`,
  and `geometry` core groups as the exact target module tree.

## Scope

- Create `models::core::card` for cards, definitions, stats, and battle class.
- Create `models::core::board` for board storage, tiles, and `BoardCard`.
- Create `models::core::geometry` for directions and positions, and keep `Player`
  at the core root.
- Create `models::session` for `Game`, `GameAction`, `GamePhase`, active-player
  state, effects, and pending work.
- Enforce the dependency direction `models::session -> models::core`.
- Give the board one checked placement operation that rejects out-of-bounds,
  blocked, and occupied positions without mutation.
- Keep local queries and invariants near the data they govern.
- Keep catalog loading under `assets`; retain the embedded CSV during Milestone 1.

## Non-goals

- No RON migration, stat-generation changes, rule extraction, frontend contract,
  dependency changes, or Cargo edits.

## Acceptance Criteria

- Core model modules do not depend on TUI, session, AI, or command/rule orchestration.
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
