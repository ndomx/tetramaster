# Repository Guidelines

## Project Shape

This is a Rust terminal implementation of Final Fantasy IX Tetra Master.

- `src/main.rs` owns the executable loop and terminal orchestration.
- `src/models/` currently contains core game data and state-machine types.
- `src/commands/` contains rule operations such as battle resolution and effect generation.
- `src/ui/` contains terminal and ASCII rendering.
- `src/assets/` contains static card records and parsing into card assets.
- `src/utils/` contains project-wide constants and small helpers.

When adding features, keep UI concerns out of game rules. Prefer moving reusable rule logic into focused modules before growing `Game`.

## Commands

- Build: `cargo build`
- Run: `cargo run`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt`

Run `cargo fmt` before finalizing Rust edits. Run `cargo test` and Clippy for behavior or module changes.

## Rust Style

- Prefer typed domain concepts over raw primitives when it clarifies intent.
- Keep randomness injectable for rule logic so tests can be deterministic.
- Prefer `Result<T, GameError>` or focused error enums over `Result<T, String>` once an area stabilizes.
- Avoid printing from domain/rule modules. Return data to the caller and let UI decide how to display it.
- Add tests around combat, effect ordering, captures, turn transitions, and board bounds before large refactors.

## Current Design Direction

`Game` is currently doing several jobs: turn state machine, player ownership, card placement, combat consequence handling, and CPU move choice. Future cleanup should split these responsibilities gradually:

- rules for pure Tetra Master behavior,
- state/session coordination,
- AI or move selection,
- terminal input/output.

Small, behavior-preserving moves are preferred over large rewrites.
