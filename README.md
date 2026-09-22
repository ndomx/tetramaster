# Tetra Master

A terminal Rust implementation of Final Fantasy IX's Tetra Master.

The game deals five cards to each side, renders a 4x4 board, and resolves captures through arrows, attack/defense stats, battle classes, and combo-style victory effects.

## Run

```sh
cargo run
```

## Development

```sh
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

## Layout

- `src/models/`: cards, board, players, positions, turn state, and game session state.
- `src/commands/`: battle resolution and effect generation.
- `src/ui/`: terminal input and ASCII rendering.
- `src/assets/`: embedded card data.
- `src/utils/`: constants, helpers, and small utility traits.

## Notes

The current implementation is intentionally small and playable from the terminal. The next useful cleanup is to move more Tetra Master rules out of `Game` and into focused rule modules that can be tested without terminal input or random global state.

## Legal notice

This is an unofficial, independently developed, non-commercial fan project. It is not affiliated with, endorsed by, sponsored by, or approved by Square Enix.

FINAL FANTASY, FINAL FANTASY IX, TETRA MASTER, and related names are trademarks or other intellectual property of their respective owners. This project contains no original game artwork, audio, source code, or other game assets. All implementation code was independently created.

The referenced names are used solely to identify the game and its associated concepts. No ownership of those names is claimed.

Any software license distributed with this repository applies only to the independently authored source code and does not grant rights to third-party trademarks or other intellectual property.
