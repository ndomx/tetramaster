# M1.11 - Verify And Document Milestone 1

## Objective

Confirm that the refactored engine and TUI satisfy the Milestone 1 contract before
any dependency, catalog-format, or Dioxus work begins.

## Prerequisites

- M1.01 through M1.10 are complete.
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
