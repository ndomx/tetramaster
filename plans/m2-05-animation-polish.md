# M2.05 - Add Event-Driven Animation And Polish

## Objective

Animate placement, combat, captures, combos, CPU turns, and game completion using
the ordered `GameUpdate` stream without moving timing into the engine.

## Prerequisites

- M2.04 is complete.
- The human has approved animation references, ordering, duration ranges,
  interrupt behavior, and reduced-motion expectations.

## Scope

- Map each approved `GameEvent` to presentation states and transitions.
- Call `advance` only after the current automatic presentation step is ready to
  continue.
- Prevent user input while an automatic transition is active, except for approved
  controls such as reduced motion or restart.
- Support reduced motion without changing event order or game state.
- Verify long combo chains, combat victory and defeat, direct capture, CPU turns,
  and game finish.

## Non-goals

- No timer or animation state in the engine, rule changes, audio unless separately
  approved, backend, or mobile work.

## Acceptance Criteria

- Every visible transition corresponds to engine events in their returned order.
- Animation cancellation or reduced motion cannot skip engine transitions.
- Input cannot race automatic progression.
- The final board and result match a non-animated run for the same seed/actions.
- Browser verification shows stable layout and correct artwork throughout motion.

## Verification

Run automated tests with animations minimized, then manually verify normal and
reduced-motion browser flows using the agreed desktop viewports.
