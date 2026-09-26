# Tetra Master Development Plan

This directory is the source of truth for the staged refactor and Dioxus web
application. The plan assumes a human is available to settle scope and design
questions before every implementation step.

## How To Run A Step

An unchecked step is planning context, not permission to implement it. At the
start of each development turn, the agent must:

1. Read `AGENTS.md`, this index, `00-decisions.md`, the selected step, and the
   relevant current code.
2. Report its understanding of the objective, prerequisites, files likely to
   change, preserved behavior, non-goals, and verification commands.
3. Ask the human about ambiguities or repository drift that materially affect the
   step. Never guess a game rule, public API, data format, or architecture.
4. Wait for the human to lock the scope and acceptance criteria before editing
   implementation files.
5. Complete only the agreed step, keep the application working, run the agreed
   checks, and report any residual risk.
6. Mark a step complete in this index only after its acceptance criteria pass.
   Record newly approved architectural decisions in `00-decisions.md`.

If a prerequisite is incomplete, the repository contradicts the plan, or a newer
human instruction changes the direction, stop and resolve that with the human.

## Architecture Direction

```text
TUI adapter ---------------\
                            +-> frontend contract -> GameSession -> rules -> models::core
future Dioxus web adapter -/                           |
                                                        +-> AI

embedded catalog -> models::core card definitions
```

- `models::core`: essential game concepts and local invariants, grouped under
  `board`, `card`, and `geometry`, with `Player` at the core root.
- `models::session`: application-level match state and coordination. It may
  depend on core models, but core models must not depend on it.
- `rules`: deterministic placement, combat, capture, and combo operations.
- `game`: session orchestration and the frontend-neutral contract.
- `ai`: CPU action selection from engine-provided legal actions.
- `tui`: terminal input, output, formatting, and optional pacing.
- `web_ui`: Dioxus-only state and RSX components, introduced in Milestone 2.
- `assets`: loading and validation of the embedded, read-only card catalog.

Milestone 1 keeps the existing Cargo dependencies and does not edit
`Cargo.toml`. It keeps `src/main.rs` as the TUI entry point while extracting a
shared library. Milestone 2 may change the manifest to introduce RON, artwork,
Dioxus, platform features, and separate binaries.

## Status

### Stage 0: Planning

- [x] Record architecture, vocabulary, frontend contract, execution workflow, and
  milestone constraints in this directory.

### Milestone 1: Engine Refactor

- [x] [M1.01 - Lock current behavior with comprehensive tests](m1-01-comprehensive-tests.md)
- [x] [M1.02 - Establish the library boundary](m1-02-library-boundary.md)
- [x] [M1.03 - Apply agreed vocabulary](m1-03-vocabulary.md)
- [x] [M1.04 - Separate core and session model responsibilities](m1-04-domain-board.md)
- [x] [M1.05 - Extract placement rules](m1-05-placement-rules.md)
- [x] [M1.06 - Extract combat, capture, and combo rules](m1-06-combat-capture.md)
- [x] [M1.07 - Centralize randomness](m1-07-randomness.md)
- [x] [M1.08 - Isolate CPU move selection](m1-08-ai.md)
- [x] [M1.09 - Implement the frontend contract](m1-09-session-contract.md)
- [x] [M1.10 - Adapt the TUI to the shared contract](m1-10-tui-adapter.md)
- [x] [M1.11 - Separate board control from player identity](m1-11-board-control.md)
- [x] [M1.12 - Verify and document Milestone 1](m1-12-verification.md)

### Milestone 2: Catalog And Dioxus Web App

- [x] [M2.01 - Migrate the catalog schema and add artwork](m2-01-catalog-schema.md)
- [x] [M2.02 - Bootstrap the Dioxus web target](m2-02-web-bootstrap.md)
- [ ] [M2.03 - Prove a playable vertical slice](m2-03-vertical-slice.md)
- [ ] [M2.04 - Build the full component and UX layer](m2-04-components-ux.md)
- [ ] [M2.05 - Add event-driven animation and polish](m2-05-animation-polish.md)
- [ ] [M2.06 - Decide whether to retain the TUI](m2-06-tui-decision.md)

## Global Guardrails

- Do not begin structural refactoring until M1.01's comprehensive baseline suite
  is complete and green.
- Preserve current game behavior during structural work. Characterize suspected
  bugs before changing them and obtain explicit approval for the behavior change.
- Keep UI concerns and framework types out of core models, rules, AI, and session code.
- Keep the TUI playable after every completed Milestone 1 step.
- Do not expose hidden CPU cards through frontend snapshots.
- Invalid actions must not partially mutate a session.
- Randomized tests must use a recorded seed.
- Update `AGENTS.md` in the same step that changes commands or repository layout;
  do not leave later agents with stale instructions.
- Do not begin a later milestone merely because an earlier task exposes related
  cleanup.

## Current Verification Baseline

Current native and browser checks:

```sh
cargo fmt --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo run --bin tetramaster-tui --features tui
cargo check --target wasm32-unknown-unknown --no-default-features --features web --bin tetramaster-web
dx serve --web --bin tetramaster-web
```

Each step lists additional checks and must update this baseline when its accepted
changes make a command obsolete.
