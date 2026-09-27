# Optional card artwork

Place developer-supplied card artwork in this directory using the snake_case
filenames declared by each entry in `src/assets/card_catalog.ron`, such as
`goblin.png` and `excalibur_ii.png`.

Images must be PNG files whose dimensions are a positive integer multiple of
84x102, such as 84x102, 168x204, or 336x408. These files are intentionally
ignored by Git. Developers are responsible for having permission to use the
artwork they provide.

Run `cargo test --all-features supplied_custom_artwork_files_are_valid` to
validate every supplied PNG's filename, catalog membership, and dimensions.
