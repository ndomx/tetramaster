# M2.04 - Build The Full Component And UX Layer

## Objective

Replace the vertical-slice presentation with a reusable, desktop-focused game UI.

## Prerequisites

- M2.03 is complete.
- The human has approved visual direction, target desktop viewport sizes, and the
  expected interactions for selection, cancellation, restart, and game over.

## Scope

- Implement `GameApp`, `Score`, `Board`, `BoardCell`, `Card`, `Hand`,
  `GameStatus`, and `GameOverDialog` or the approved equivalents.
- Use one card component for hand and board contexts.
- Render the bundled fallback artwork and Tetra Master stats and arrows clearly.
- Distinguish empty, blocked, occupied, selected, legal, and disabled states
  without relying on color alone.
- Provide keyboard-accessible controls and visible focus states.
- Keep the 4x4 board stable and prevent text, cards, or controls from resizing or
  overlapping during state changes.

## Non-goals

- No mobile-specific application, server, persistence, gameplay-rule changes, or
  final combat animation choreography.
- Canonical artwork loading is deferred to M2.04.1.
- Contested-card previews are deferred to M2.04.2 so the web UI does not
  duplicate interaction-discovery rules.

## Acceptance Criteria

- All ordinary player workflows are available without coordinate entry.
- The longest card names and all stat displays fit supported desktop viewports.
- Ownership and legal interactions remain understandable without color.
- Component tests cover important visual states and commands.
- Browser screenshots show no blank fallback assets, overflow, or overlap.

## Verification

Run the full build/test/lint matrix and capture browser screenshots at every
agreed desktop viewport. Inspect the fallback artwork and interactive states.
