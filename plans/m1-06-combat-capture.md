# M1.06 - Extract Combat, Capture, And Combo Rules

## Objective

Move combat calculation, ownership changes, and combo discovery into focused,
testable rule modules while preserving characterized behavior.

## Prerequisites

- M1.05 is complete.
- Before editing, the human has resolved the intended combo traversal and ordering
  or explicitly approved preserving the characterized implementation.

## Scope

- Extract combat calculation from the old command/session code.
- Separate direct capture, combat-result capture, and combo propagation concepts.
- Keep internal pending effects distinct from future frontend events.
- Return typed outcomes that session orchestration can apply.
- Remove the old `commands` module only after all callers and tests have moved.

## Non-goals

- No RNG redesign beyond parameters strictly needed for extraction, no AI work,
  no frontend event API, no dependency changes, and no Cargo edits.

## Acceptance Criteria

- Combat, capture, and combo rules are testable without `GameSession` or a TUI.
- Rule modules do not print or own presentation timing.
- Effect ordering is explicit and covered by tests.
- Existing game behavior and TUI playability are preserved.
- `AGENTS.md` is updated when `commands` is removed.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```
