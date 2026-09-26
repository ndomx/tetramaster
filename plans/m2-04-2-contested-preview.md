# M2.04.2 - Preview Contested Cards

## Objective

Preview which opposing cards would be contested when the player targets a legal
board cell, without duplicating game rules in the web adapter, and expose the
resolved attack and defense power when combat begins.

## Prerequisites

- M2.04 is complete.
- The human has approved preview timing for pointer and keyboard interaction.
- The frontend-neutral query shape has been agreed before implementation.

## Scope

- Add the smallest frontend-neutral session or rules query needed to discover
  interactions for a selected card and legal destination.
- Render a non-color-only contested state for the returned opposing cards.
- Support both pointer hover and keyboard focus.
- Add deterministic tests proving the preview agrees with placement interaction
  discovery and never mutates the session.
- Extend the combat event contract with the already-calculated attack and defense
  power used for that resolution, including the applied random calculation.
- Show those resolved powers on the two combatants once combat begins. This is
  historical event presentation, not a pre-combat prediction.

## Non-goals

- No predicted combat winner, pre-combat random-roll preview, mutation, or final
  animation choreography.

## Acceptance Criteria

- The web UI consumes the shared interaction query and contains no copied combat
  or arrow-adjacency rules.
- Pointer and keyboard users receive the same preview.
- Preview queries leave the session and random state unchanged.
- Combat power values shown by the frontend exactly match the values used by the
  engine to resolve that combat.
