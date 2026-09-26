# Optional card artwork

Place developer-supplied card artwork in this directory using the canonical
one-based names `Card001.png` through `Card100.png`.

Images must be PNG files whose dimensions are a positive integer multiple of
84x102, such as 84x102, 168x204, or 336x408. These files are intentionally
ignored by Git. Developers are responsible for having permission to use the
artwork they provide.

Run `cargo test --all-features supplied_custom_artwork_files_are_valid` to
validate every supplied PNG's filename and dimensions.
