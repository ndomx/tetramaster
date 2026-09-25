# M1.12 - Verify And Document Milestone 1

## Objective

Confirm that the refactored engine and TUI satisfy the Milestone 1 contract before
any dependency, catalog-format, or Dioxus work begins.

## Prerequisites

- M1.01 through M1.11 are complete.
- No unresolved human decision blocks the public contract.

## Scope

- Audit module dependency direction and public visibility.
- Remove obsolete helpers and compatibility shims only when tests prove them
  unused.
- Confirm core models, rules, game, and AI code contain no Crossterm, terminal I/O,
  presentation delays, or Dioxus concepts.
- Run the complete native verification matrix and manually play a seeded terminal
  match.
- Update `README.md`, `AGENTS.md`, and this index to the actual layout and commands.
- Record intentional rule deviations and residual risks.

## Cargo Constraint

Do not edit `Cargo.toml`. A true WebAssembly build and target-specific dependency
split are deferred to M2.02. This step verifies source-level frontend isolation,
not final browser compilation.

## Acceptance Criteria

- Every Milestone 1 step meets its own acceptance criteria.
- The TUI is fully playable through the public frontend contract.
- A seeded test adapter can complete a game deterministically.
- No frontend decides outcomes, mutates engine internals, or selects CPU moves.
- Repository guidance matches reality.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```

## Completed Verification

Milestone 1 was verified against the current frontend contract after removing
the pre-contract execution path and other obsolete compatibility code.

- `cargo fmt --check`: passed.
- `cargo test`: passed (44 tests).
- `cargo clippy --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- Source audit: core models, session coordination, rules, AI, assets, and
  utilities contain no Crossterm, terminal I/O, presentation delays, Dioxus
  concepts, or UI dependencies.
- Output audit: every remaining `print!`, `println!`, or `eprintln!` call is in
  the terminal UI adapter. Gameplay status is rendered from `GameEvent` values;
  prompts and invalid-input messages remain UI-local.
- Legacy audit: the obsolete `Game` alias, `TurnResult`, `ActivePlayer`,
  `TileCard`, legacy turn runner, stale tile-card abbreviations, and unused
  compatibility helpers are absent.
- Manual TUI playthrough: a complete match was played through the public
  `dispatch`/`advance` contract and finished normally with a 5-4 player result.
- Deterministic adapter coverage: seeded contract tests complete games and
  reproduce snapshots and event sequences for identical seeds and actions.

## Intentional Behavior And Residual Risks

- The current human player is assigned Blue and the CPU Red. Starting-player
  selection remains random; future PvP color assignment is intentionally
  deferred.
- Multiple battles retain their characterized deterministic processing order.
- Combo discovery remains one hop. Each combo capture is emitted as an
  individual control-change transition even though the original game presents a
  combo's captures simultaneously.
- `CaptureKind` remains explicit rule context by decision, although current
  capture behavior does not branch on it.
- Removing obsolete player-ID random draws changes the sequences produced by
  historical seeds. Runs remain deterministic for a given seed and action
  sequence under the current implementation.
- The TUI does not print its generated seed. Replayability is verified through
  seeded adapters and tests rather than debug output.
- Frontend isolation is verified at the Rust source boundary only. A true
  WebAssembly build and target-specific dependency split remain deferred to
  M2.02 under the Milestone 1 Cargo constraint.
