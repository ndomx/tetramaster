# M1.10 - Adapt The TUI To The Shared Contract

## Objective

Make the existing terminal application the first real adapter for exactly the
contract the later Dioxus app will consume.

## Prerequisites

- M1.09 is complete and its public contract is covered by integration tests.
- The human has approved how much terminal UX cleanup belongs in this step.

## Scope

- Keep `src/main.rs` as the entry point and preserve `cargo run`.
- Move terminal-specific presentation under `tui` modules where useful without
  changing `Cargo.toml`.
- Render only `GameSnapshot` and newly returned `GameEvent` values.
- Convert terminal prompts into one complete `GameAction` before dispatch.
- Follow `InteractionState`: prompt, call `advance`, or render the result.
- Keep blocking input, terminal colors, formatting, and optional event pacing in
  the adapter.
- Present `GameError` and retry instead of panicking or discarding failures.
- Consolidate duplicated hand/board card ASCII rendering where it reduces real
  duplication without changing the engine.

## Non-goals

- No raw-keyboard redesign, Dioxus code, changes to the public engine contract,
  dependency changes, or Cargo edits.

## Acceptance Criteria

- TUI code does not read or mutate `GameSession` internals.
- The TUI never calls rule or AI functions directly.
- Automatic progress uses `advance`, not a fixed polling loop.
- Events are consumed in returned order, and presentation delays do not alter game
  state.
- A complete terminal game can be played without an unwrap-driven crash.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```

Update `AGENTS.md` in this step if TUI paths or commands change.
