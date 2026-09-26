# M2.04.1 - Load Canonical Card Artwork

## Objective

Replace the shared fallback image with each card's canonical artwork while
retaining a reliable local fallback.

## Prerequisites

- M2.04 is complete.
- The human has approved whether artwork is resolved remotely, vendored, or
  generated from another authoritative source.

## Scope

- Resolve the catalog's canonical Fandom page references to browser-loadable
  artwork using the approved strategy.
- Keep the existing 84x102 dimensions and local fallback behavior.
- Prevent missing or delayed artwork from shifting the card layout.
- Verify all 100 catalog entries rather than relying on random dealt hands.

## Non-goals

- No card component redesign, gameplay changes, persistence, or animation work.

## Acceptance Criteria

- Every catalog card resolves to its intended artwork or the local fallback.
- Failed artwork requests do not produce blank cards or layout shifts.
- Automated checks cover URL resolution and fallback behavior.
