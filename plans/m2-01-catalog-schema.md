# M2.01 - Migrate The Catalog Schema And Add Artwork

## Objective

Replace the interim compact CSV with the approved extensible RON catalog and add
canonical artwork metadata before building the web UI.

## Prerequisites

- Milestone 1 is complete.
- The human has approved the artwork source, licensing, file format, dimensions,
  repository location, naming convention, and missing-art fallback.
- The human has approved the initial RON schema and required versus optional
  fields.

## Scope

- Add the RON dependency and remove CSV only after migration tests pass.
- Convert every existing catalog record without changing its canonical values.
- Store stats as named expanded fields and battle classes as typed variants.
- Add artwork metadata to `CardDefinition` without putting decoded image data in
  the core card model.
- Keep the catalog embedded, immutable, and initialized once.
- Validate unique definitions, required fields, stat ranges, and artwork paths
  with entry-specific errors.
- Verify the initial migration before deleting the CSV; no permanent parallel
  catalog or migration fixture is retained once RON becomes definitive.

## Non-goals

- No runtime catalog editing, remote asset service, card balance changes, RSX
  component work, or unrelated dependency upgrades.

## Acceptance Criteria

- RON is the sole canonical catalog source and remains human-readable.
- Every definition has valid artwork metadata or the explicitly approved fallback.
- Invalid embedded data fails at startup with useful context.
- Existing seeded engine tests produce equivalent gameplay results.
- `README.md` and `AGENTS.md` describe the new catalog location and validation.

## Verification

Agree on the post-manifest command matrix at kickoff, then at minimum run format,
all tests, Clippy with all enabled features, and a TUI smoke test.
