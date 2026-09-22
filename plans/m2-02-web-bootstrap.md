# M2.02 - Bootstrap The Dioxus Web Target

## Objective

Add a client-rendered Dioxus browser target while preserving the shared library
and working TUI.

## Prerequisites

- M2.01 is complete.
- The human has approved the Dioxus version, feature layout, binary names, and
  development/deployment commands after the agent checks current official docs.

## Scope

- Add Dioxus with only the features needed for a browser client.
- Separate web and TUI binaries and platform-specific dependencies in
  `Cargo.toml`.
- Make Crossterm TUI-only and keep it out of the WebAssembly dependency graph.
- Add the minimal Dioxus configuration and a web entry point that renders a static
  application shell.
- Verify the shared library and web binary for `wasm32-unknown-unknown`.
- Preserve a documented command for launching the TUI.

## Non-goals

- No Fullstack, SSR, backend, mobile target, router, gameplay components, or public
  engine API changes.

## Acceptance Criteria

- The empty web application starts in a desktop browser.
- The TUI remains runnable through its documented command.
- The web build does not compile Crossterm.
- Domain, rules, AI, and session modules contain no Dioxus types.
- `README.md` and `AGENTS.md` contain exact native and web commands.

## Verification

The kickoff must record exact commands for formatting, all-feature tests and
Clippy, the TUI run, the WebAssembly check, and `dx serve --web` or its approved
current equivalent.
