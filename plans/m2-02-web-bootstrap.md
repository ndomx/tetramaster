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

## Completion Notes

- Dioxus 0.7.10 is exposed through a browser-only `web` feature; Crossterm is an
  optional native-only dependency behind the default `tui` feature.
- The native and browser binaries are `tetramaster-tui` and `tetramaster-web`.
- The static browser shell uses the Dioxus-managed Tailwind pipeline without a
  router, Fullstack, SSR, backend, desktop, or mobile feature.
- `cargo fmt --check`, `cargo test --all-features`, all-target/all-feature
  Clippy with warnings denied, and the explicit WebAssembly check pass.
- The WebAssembly dependency graph contains no Crossterm package.
- `dx serve --web --bin tetramaster-web` built and served the shell at
  `http://127.0.0.1:8080`; a browser rendered the title, shell text, and generated
  Tailwind styling.
