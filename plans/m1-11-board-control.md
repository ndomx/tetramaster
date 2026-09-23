# M1.11 - Separate Board Control From Player Identity

## Objective

Replace numeric player IDs in board state and rules with a typed, match-local
concept representing which side currently controls a card.

Board control changes when a card is captured. It must not imply any broader or
long-lived ownership of that card, and the rules must not require a player's
identity merely to distinguish the two sides in a match.

## Prerequisites

- M1.10 is complete and green.
- The human has approved the name of the typed board-control concept and its two
  variants.

## Open Question

- What should the typed board-control concept and its two variants be called?

Do not choose the names silently during implementation. The names must describe
the two match-local sides without encoding whether either side is human- or
CPU-controlled and without using vague ordinal labels.

## Scope

- Introduce the approved typed representation for the current controller of a
  board card.
- Replace `BoardCard::owner_id` and numeric ownership parameters, comparisons,
  capture outcomes, and score queries with that type.
- Remove `Player::id` if it has no remaining responsibility after the change.
- Update session orchestration, rules, frontend snapshots and events, and the TUI
  adapter to use the typed board-control concept consistently.
- Update tests to retain all characterized placement, combat, capture, combo,
  scoring, and rendering behavior.

## Non-goals

- No new player identity model, persistence, accounts, networking, game modes,
  dependencies, catalog changes, or UI redesign.
- No changes to capture, combat, combo, scoring, or turn behavior.
- No changes to runtime card identity unless separately approved.

## Acceptance Criteria

- Board state and rules do not use a numeric player ID to represent current card
  control.
- Capturing a card changes only its match-local controller.
- Scoring and opposing-card checks use the typed controller.
- The frontend contract exposes the typed controller without leaking session
  implementation details.
- `Player` contains no identifier that exists solely to support board ownership.
- Existing behavior-locking tests remain green, with assertions updated only for
  the approved type-level representation.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```
