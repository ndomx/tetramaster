# M1.01 - Establish The Library Boundary

## Objective

Create a shared Rust library target without changing behavior, paths beyond what
is required, dependencies, or the terminal launch command.

## Prerequisites

- Stage 0 is complete.
- The human has approved the exact module visibility for this step.

## Kickoff Questions

- Confirm whether modules should initially be broadly `pub` for migration or
  exposed through a narrow temporary prelude.
- Confirm whether any unrelated repository changes overlap `main.rs` or module
  declarations.

## Scope

- Add `src/lib.rs` as the single owner of crate module declarations.
- Change `src/main.rs` to import the package library instead of declaring a
  duplicate module tree.
- Preserve `cargo run` as the TUI command.
- Confirm the library can be imported by integration tests.

## Non-goals

- No renames, module moves, rule changes, new abstractions, dependency changes, or
  `Cargo.toml` edits.

## Acceptance Criteria

- Domain types are not compiled separately for the library and binary.
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
