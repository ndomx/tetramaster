# M1.05 - Extract Placement Rules

## Objective

Separate placement legality and neighboring interaction discovery from session
coordination.

## Prerequisites

- M1.04 is complete.
- The human has approved whether legal actions are represented temporarily or
  whether the final `GameAction` type should be introduced now.

## Scope

- Add a focused placement rule module.
- Determine legal positions and placement-triggered neighboring interactions from
  explicit inputs.
- Return values describing results; do not print, choose moves, mutate unrelated
  state, or know about terminal input.
- Route the existing session through the extracted placement rules while
  preserving behavior.

## Non-goals

- No combat resolution, combo traversal changes, AI extraction, public frontend
  contract, dependency changes, or Cargo edits.

## Acceptance Criteria

- Placement rules can be tested without constructing a terminal.
- Board mutation remains checked and failed operations remain atomic.
- Existing placement and effect-discovery behavior is unchanged.
- No rule module imports `ui`, Crossterm, or standard I/O.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
