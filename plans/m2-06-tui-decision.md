# M2.06 - Decide Whether To Retain The TUI

## Objective

Make an explicit human decision about the optional terminal frontend after the
web application reaches parity.

## Prerequisites

- M2.05 is complete.
- The agent has measured the TUI's dependency, testing, documentation, and
  maintenance cost without deleting or rewriting it.

## Decision Inputs

- Does the TUI still consume only the shared frontend contract?
- Does it materially complicate Cargo features, CI, or dependency upgrades?
- Is it useful as a debugging, accessibility, or lightweight play mode?
- Does it have tests sufficient to prevent silent breakage?

## Outcomes

- If retained, document it as an optional supported binary and keep its build and
  smoke checks in the standard verification matrix.
- If removed, obtain explicit human approval, remove only TUI-specific code and
  dependencies, and retain all shared engine tests.

## Non-goals

- Do not redesign the terminal UX merely to justify retaining it.
- Do not remove the TUI as incidental cleanup during another web step.

## Acceptance Criteria

- The decision and rationale are recorded in `00-decisions.md`.
- Commands, features, README, AGENTS guidance, and CI match the chosen outcome.
- The web frontend and shared engine remain unaffected by the decision.
