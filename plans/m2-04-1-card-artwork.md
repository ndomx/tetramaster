# M2.04.1 - Load Canonical Card Artwork

## Objective

Resolve optional developer-supplied artwork for each card while retaining a
reliable bundled fallback.

## Prerequisites

- M2.04 is complete.
- The human has approved an optional, Git-ignored local artwork pack.

## Scope

- Resolve `Card001.png` through `Card100.png` from `assets/cards/custom/` using
  each snapshot's canonical definition index.
- Accept PNG source dimensions that are positive integer multiples of 84x102.
- Keep the existing 84x102 dimensions and local fallback behavior.
- Prevent missing or delayed artwork from shifting the card layout.
- Verify all 100 catalog entries rather than relying on random dealt hands.

## Non-goals

- No card component redesign, gameplay changes, persistence, or animation work.

## Acceptance Criteria

- Every catalog card resolves to its developer-supplied artwork or the local
  fallback.
- Failed artwork requests do not produce blank cards or layout shifts.
- Automated checks cover URL resolution and fallback behavior.
