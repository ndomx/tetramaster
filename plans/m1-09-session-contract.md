# M1.09 - Implement The Frontend Contract

## Objective

Reduce `Game` to `GameSession` orchestration and expose the only public contract
that both the TUI and future Dioxus frontend may use.

## Prerequisites

- M1.08 is complete.
- The human has resolved tie results and runtime card identity.
- The human has approved the exact event variants, payloads, and transition
  granularity proposed during kickoff.

## Required Contract

- `GameAction`, initially `PlayCard { card_id, position }`.
- `GameEvent` for observable state changes, separate from pending effects.
- `InteractionState`: `AwaitingPlayerAction`, `Advancing`, or `Finished`.
- Owned `GameSnapshot: Clone + PartialEq`, containing visible state and legal human
  actions but no hidden CPU cards or mutable internals.
- `GameUpdate`, containing ordered events, the post-event snapshot, and resulting
  interaction state.
- Typed `GameError` values.

## Transition Invariants

- `dispatch` is valid only during `AwaitingPlayerAction`.
- `advance` performs one automatic player-visible transition.
- Each successful call returns one `GameUpdate`; its snapshot is the state after
  every returned event.
- Internal bookkeeping may be collapsed only until the next observable event or
  stable interaction state.
- Invalid operations leave the complete session unchanged.
- Frontends never dispatch CPU actions; `advance` asks the AI and applies its
  result through the shared internal validator.

## Scope

- Implement session creation, `snapshot`, `dispatch`, and `advance` around those
  invariants.
- Route placement, combat, capture, combo, AI, and turn progression through their
  established owners.
- Add seeded integration tests that drive complete games solely through the public
  contract and assert important event sequences.

## Non-goals

- No TUI rewrite, Dioxus types, animation timing, catalog format migration,
  dependency changes, or Cargo edits.

## Acceptance Criteria

- A test adapter can play a full deterministic match without accessing session
  fields or rule functions.
- Snapshots never reveal CPU cards.
- Event order and state-after-event semantics are tested.
- Invalid action atomicity is tested.
- Session code coordinates but does not implement rules or AI policy.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
