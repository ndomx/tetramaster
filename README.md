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
