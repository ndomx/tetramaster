# Tetra Master

[![CI](https://github.com/ndomx/tetramaster/actions/workflows/ci.yml/badge.svg)](https://github.com/ndomx/tetramaster/actions/workflows/ci.yml)

A terminal Rust implementation of Final Fantasy IX's Tetra Master, with a
frontend-neutral game engine shared through the library crate.

The game deals five cards to each side, renders a 4x4 board, and resolves captures through arrows, attack/defense stats, battle classes, and combo-style victory effects.

## Run

```sh
cargo run
```

## Development

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Layout

- `src/models/core/`: cards, board state, players, and geometry.
- `src/models/session/`: `GameSession` orchestration and the frontend contract.
- `src/rules/`: placement, combat, capture, and combo rules.
- `src/ai/`: CPU move selection from engine-provided legal actions.
- `src/ui/`: terminal input and ASCII rendering.
- `src/assets/`: the embedded, read-only card catalog.
- `src/utils/`: constants, coordinate helpers, and seeded randomness.

## Architecture

The terminal adapter drives `GameSession` exclusively through snapshots,
`GameAction`, `dispatch`, and `advance`. Core models and rules contain no terminal
I/O or Crossterm dependencies. Each match owns a seeded random generator, and the
terminal prints its seed when play begins.

The current player controls Blue cards and the CPU controls Red cards. Color is
independent of the randomly selected starting player.

## Legal notice

This is an unofficial, independently developed, non-commercial fan project. It is not affiliated with, endorsed by, sponsored by, or approved by Square Enix.

FINAL FANTASY, FINAL FANTASY IX, TETRA MASTER, and related names are trademarks or other intellectual property of their respective owners. This project contains no original game artwork, audio, source code, or other game assets. All implementation code was independently created.

The referenced names are used solely to identify the game and its associated concepts. No ownership of those names is claimed.

Any software license distributed with this repository applies only to the independently authored source code and does not grant rights to third-party trademarks or other intellectual property.
