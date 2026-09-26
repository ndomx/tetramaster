# Repository Guidelines

## Project Shape

This is a Rust terminal implementation of Final Fantasy IX Tetra Master.

- `src/lib.rs` owns the shared crate module declarations.
- `src/bin/tui.rs` owns the executable loop and terminal orchestration, importing
  shared code through the library crate. It requires the native `tui` feature.
- `src/bin/web.rs` launches the client-rendered browser adapter. Dioxus components
  live under `src/web_ui/` and require the `web` feature.
- `src/models/core/` contains the essential game model, grouped into `board`,
  `card`, and `geometry`, plus the player model. Core models own only their local
  invariants and must not depend on session, rules, AI, or UI code.
- `src/models/session/` contains application-level match state and coordination
  types such as `GameSession`, `GameAction`, `GamePhase`, and pending effects.
  Session models may depend on core models, never the reverse.
- `src/rules/` contains frontend-neutral placement, combat, capture, and combo
  rule logic.
- `src/ui/` contains terminal and ASCII rendering.
- `tailwind.css` is the Tailwind input; `dx` compiles it to
  `assets/tailwind.css` for the browser adapter.
- `src/assets/card_catalog.ron` is the canonical embedded card catalog;
  `src/assets/` owns its parsing and validation into card definitions. Catalog
  order is the zero-based runtime index. Artwork metadata links to the external
  one-based `Card001.png` through `Card100.png` references. The bundled local
  artwork is `assets/cards/fallback.png`; optional developer-provided PNGs live
  under the Git-ignored `assets/cards/custom/` directory.
- `src/utils/` contains project-wide constants and small helpers.

Catalog changes must preserve unique names and artwork URLs, 84x102 artwork
dimensions, expanded stat tiers whose low nibble is `0xF`, and typed battle
classes. RON is the definitive catalog; do not add a parallel CSV record.
Custom artwork may use any positive integer multiple of 84x102 and must retain
the same aspect ratio.

When adding features, keep UI concerns out of game rules. Prefer moving reusable
rule logic into focused modules before growing `GameSession`.

## Commands

- Build TUI: `cargo build --bin tetramaster-tui --features tui`
- Run TUI: `cargo run --bin tetramaster-tui --features tui`
- Serve web: `dx serve --web --bin tetramaster-web`
- Check web: `cargo check --target wasm32-unknown-unknown --no-default-features --features web --bin tetramaster-web`
- Test: `cargo test --all-features`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Format: `cargo fmt`

Run `cargo fmt` before finalizing Rust edits. Run `cargo test` and Clippy for behavior or module changes.

## Human-Guided Refactor Workflow

`plans/README.md` is the plan index, and `plans/00-decisions.md` records settled
architecture. The individual files under `plans/` define agent-sized steps. They
are planning context, not authorization to implement every unchecked item at
once.

When starting a refactor step:

1. Read this file, `plans/README.md`, `plans/00-decisions.md`, the selected step,
   and the relevant current implementation before proposing changes.
2. Summarize your understanding of the selected step, including its boundaries,
   likely files, behavior that must remain unchanged, and intended verification.
3. Identify ambiguities, contradictions, and decisions that affect that step.
   Ask the human to resolve them before editing code; do not silently choose game
   rules, public APIs, data formats, or architecture.
4. Agree on the step's deliverables and acceptance checks with the human. Do not
   expand into later steps or milestones without explicit approval.
5. Implement incrementally and keep the TUI playable after each completed step
   unless the agreed scope explicitly says otherwise.
6. Run the agreed checks, report their results, and update the relevant plan
   checklist only after the work satisfies its acceptance criteria.

If the plan conflicts with the repository or a newer human instruction, stop and
ask which source should govern the step. Decisions recorded under established
decisions remain authoritative until the human explicitly changes them.

The project remains one Cargo package with a shared library and separate TUI and
web binaries. Crossterm must stay native-only and Dioxus types must stay inside
the web adapter. Do not add Fullstack, SSR, backend, router, desktop, or mobile
features unless a later approved plan step requires them.

Use Dioxus 0.7 APIs. Components use owned props and `Element`, local reactive
state uses `use_signal`, and static assets use `asset!`. Do not use removed APIs
such as `Scope`, `cx`, or `use_state`. Do not hold Dioxus signal read or write
guards across an `await` point; `clippy.toml` treats those guard types as invalid
across awaits.

Do not begin structural refactoring until M1.01's comprehensive characterization
suite is complete and passing. Later steps must preserve those tests; changing a
baseline assertion requires explicit human approval of the behavior change.

## Rust Style

- Prefer typed core game concepts over raw primitives when it clarifies intent.
- Keep randomness injectable for rule logic so tests can be deterministic.
- Prefer `Result<T, GameError>` or focused error enums over `Result<T, String>` once an area stabilizes.
- Avoid printing from core model or rule modules. Return data to the caller and let UI decide how to display it.
- Add tests around combat, effect ordering, captures, turn transitions, and board bounds before large refactors.

## Current Design Direction

`GameSession` coordinates the turn state machine and translates rule outcomes
into the frontend contract. Board invariants, placement/combat/capture/combo
rules, CPU move selection, and terminal presentation live in their own modules:

- rules for pure Tetra Master behavior,
- session coordination and frontend events,
- AI or move selection,
- terminal input/output.

Small, behavior-preserving moves are preferred over large rewrites.
