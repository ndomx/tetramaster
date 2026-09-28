# Tetra Master

[![CI](https://github.com/ndomx/tetramaster/actions/workflows/ci.yml/badge.svg)](https://github.com/ndomx/tetramaster/actions/workflows/ci.yml)

A Rust implementation of Final Fantasy IX's Tetra Master, with a
frontend-neutral game engine shared by terminal and browser adapters.

The game deals five cards to each side, renders a 4x4 board, and resolves captures through arrows, attack/defense stats, battle classes, and combo-style victory effects.

## Run The Terminal App

```sh
cargo run --bin tetramaster-tui --features tui
```

## Run The Web App

Install the Dioxus CLI and the WebAssembly target once, then launch the browser
development server. Dioxus downloads and runs the Tailwind compiler when needed.

```sh
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10
dx serve --web --bin tetramaster-web
```

## Development

```sh
cargo fmt --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo check --target wasm32-unknown-unknown --no-default-features --features web --bin tetramaster-web
```

## Layout

- `src/models/core/`: cards, board state, players, and geometry.
- `src/models/session/`: `GameSession` orchestration and the frontend contract.
- `src/rules/`: placement, combat, capture, and combo rules.
- `src/ai/`: CPU move selection from engine-provided legal actions.
- `src/bin/tui.rs`: native terminal entry point.
- `src/bin/web.rs`: browser entry point.
- `src/ui/`: terminal input and ASCII rendering, enabled by the `tui` feature.
- `src/web_ui/`: Dioxus components, enabled by the `web` feature.
- `src/assets/card_catalog.ron`: the embedded, read-only card catalog. Definitions
  use named expanded stats, typed battle classes, and external artwork references.
- `assets/cards/fallback.png`: the original 84x102 image used when external card
  artwork is unavailable. Third-party card images are not stored in this repository.
- `assets/cards/custom/`: optional Git-ignored developer artwork using the
  snake_case filenames declared in the catalog. PNG dimensions may be any
  positive integer multiple of 84x102; run
  `cargo test --all-features supplied_custom_artwork_files_are_valid` to validate
  a supplied pack.
- `tailwind.css`: Tailwind input compiled by Dioxus into `assets/tailwind.css`.
- `src/utils/`: constants, coordinate helpers, and seeded randomness.

## Architecture

Frontend adapters drive `GameSession` exclusively through snapshots,
`GameAction`, `dispatch`, and `advance`. Core models and rules contain no terminal,
Crossterm, or Dioxus dependencies. Each match owns a seeded random generator, and
observable game progress is presented from `GameEvent` values.

The current player controls Blue cards and the CPU controls Red cards. Color is
independent of the randomly selected starting player.

The catalog is parsed and validated once at startup. Its zero-based order remains
the runtime definition index, while each entry independently declares the
snake_case PNG filename used for artwork lookup.

## Deploy The Web App

Build the static browser application with:

```sh
dx build --web --release --bin tetramaster-web
```

Serve `target/dx/tetramaster-web/release/web/public` over HTTP with any static
hosting provider or web server. Opening `index.html` directly through `file://`
is not supported by browser WebAssembly loading.

Production artwork is deliberately separate from the application bundle. Route
the same-origin `/cards/` URL prefix to a CDN, object store, or static directory
that contains the catalog-declared filenames. For example,
`/cards/goblin.png` must return the deployed `goblin.png`. The hosting layer may
implement that route however it chooses; no provider-specific service is required.
Missing or unreachable artwork uses the bundled fallback image.

The included Dockerfile and Caddyfile are optional, provider-neutral examples.
The Caddy example serves the application from `/srv` and reads `/cards/*` from
`/artwork`; adapt or replace those paths for the chosen hosting environment.

## Legal notice

This is an unofficial, independently developed, non-commercial fan project. It is not affiliated with, endorsed by, sponsored by, or approved by Square Enix.

FINAL FANTASY, FINAL FANTASY IX, TETRA MASTER, and related names are trademarks or other intellectual property of their respective owners. This project contains no original game artwork, audio, source code, or other game assets. All implementation code was independently created.

The referenced names are used solely to identify the game and its associated concepts. No ownership of those names is claimed.

Any software license distributed with this repository applies only to the independently authored source code and does not grant rights to third-party trademarks or other intellectual property.
