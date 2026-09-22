# M1.02 - Establish The Library Boundary

## Objective

Create a shared Rust library target while preserving the comprehensive M1.01 suite
and all working terminal behavior.

## Prerequisites

- M1.01 is complete, comprehensive, and green.
- The human has approved the exact module visibility for this step.

## Kickoff Questions

- Confirm whether modules should initially be broadly `pub` for migration or
  exposed through a narrow temporary API.
- Confirm the placement of existing tests after module declarations move.
- Confirm whether unrelated repository changes overlap `main.rs`, tests, or module
  declarations.

## Scope

- Add `src/lib.rs` as the single owner of crate module declarations.
- Change `src/main.rs` to import the package library instead of declaring a
  duplicate module tree.
- Move or adjust test modules only as required by the new crate boundary; preserve
  their scenarios and assertions.
- Preserve `cargo run` as the TUI command.
- Confirm the library can be imported by later integration tests.

## Non-goals

- No weakened/deleted baseline assertions, renames, module reorganization beyond
  the crate boundary, rule changes, new abstractions, dependency changes, or
  `Cargo.toml` edits.

## Acceptance Criteria

- Domain types are not compiled separately for the library and binary.
- Every M1.01 test still covers the same behavior and passes.
- Any necessary test relocation is mechanical and reviewable.
- The terminal game starts and behaves as before.
- Existing build, test, and lint checks pass.

## Verification

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run
```

Update `AGENTS.md` in this step if the documented entry-point ownership changes.
