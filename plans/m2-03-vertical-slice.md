# M2.03 - Prove A Playable Vertical Slice

## Objective

Prove the Dioxus adapter against the existing engine contract before building the
full component system.

## Prerequisites

- M2.02 is complete.
- The human has approved the minimal visual scope and browser verification method.

## Scope

- Store `GameSession` in Dioxus state without changing its public API.
- Render a minimal score, human hand, and 4x4 board from `GameSnapshot`.
- Keep selected-card state in the web adapter.
- Enable choices from snapshot legal actions and dispatch one complete
  `GameAction::PlayCard` from clicks.
- Consume `GameUpdate`, render basic event/status text, and schedule `advance`
  while the interaction state is `Advancing`.
- Complete a full match even if the presentation is deliberately plain.

## Non-goals

- No final styling, animation choreography, router, persistence, backend, mobile
  layout, or engine contract redesign for frontend convenience.

## Acceptance Criteria

- A complete match is playable in a desktop browser solely through the shared
  contract.
- Web code does not inspect or mutate session internals.
- Hidden CPU cards remain hidden.
- Errors are shown without losing the session.
- An integration test covers selection-to-dispatch wiring.

## Verification

Run the full native/web command matrix and manually verify a complete browser
match at the desktop viewports agreed during kickoff.
