# M1.03 - Apply Agreed Vocabulary

## Objective

Perform behavior-neutral renames so later agents work with stable domain terms.

## Prerequisites

- M1.02 is complete and green.
- The human has reviewed the complete rename list against the current repository.

## Required Renames

- `CardAsset` -> `CardDefinition`
- `TileCard` -> `BoardCard`
- `Tile::Card` -> `Tile::Occupied`
- `Tile::Block` -> `Tile::Blocked`
- TUI views to `CardTileView`, `BlockedTileView`, and `EmptyTileView`
- `EffectInstance` -> `PendingEffect`
- `Effect::Capture` -> `Effect::DirectCapture`
- `GameState` -> `GamePhase`
- `find_available` -> `available_position`
- `find_placed_by_id` -> `position_of_card`

The existing action is renamed to `GameAction` as a struct without prematurely
implementing the M1.09 enum contract. M1.09 must still settle the final public
action shape.

## Scope

- Apply approved renames mechanically across source and tests.
- Replace read-only `&Vec<T>` APIs with slices where this is purely mechanical.
- Remove only methods proven unused after checking source and tests.

## Non-goals

- No module relocation, behavior changes, catalog format changes, contract design,
  dependency changes, or Cargo edits.

## Acceptance Criteria

- No old term remains except compatibility notes or historical documentation.
- The diff is mechanical and existing behavior tests remain unchanged and green.
- `AGENTS.md` is updated if it names renamed paths or concepts.

## Verification

```sh
rg "CardAsset|TileCard|EffectInstance|GameState|find_placed_by_id" -g '*.rs' .
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

The `rg` command is expected to produce no matches after approved compatibility
references are excluded.
