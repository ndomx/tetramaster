use std::fmt;

pub const CARD_ARTWORK_WIDTH: u32 = 84;
pub const CARD_ARTWORK_HEIGHT: u32 = 102;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkSources {
    pub primary_url: String,
    pub fallback_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtworkLoadState {
    Primary,
    Fallback,
}

impl ArtworkLoadState {
    pub fn source(self, sources: &ArtworkSources) -> &str {
        match self {
            Self::Primary => &sources.primary_url,
            Self::Fallback => &sources.fallback_url,
        }
    }

    pub fn use_fallback(&mut self) {
        *self = Self::Fallback;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArtworkResolver<'a> {
    custom_base_url: &'a str,
    fallback_url: &'a str,
}

impl<'a> ArtworkResolver<'a> {
    pub fn new(custom_base_url: &'a str, fallback_url: &'a str) -> Self {
        Self {
            custom_base_url: custom_base_url.trim_end_matches('/'),
            fallback_url,
        }
    }

    pub fn resolve(self, filename: &str) -> Result<ArtworkSources, ArtworkError> {
        if !is_valid_artwork_filename(filename) {
            return Err(ArtworkError::InvalidFilename(filename.to_owned()));
        }

        Ok(ArtworkSources {
            primary_url: format!("{}/{filename}", self.custom_base_url),
            fallback_url: self.fallback_url.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtworkError {
    InvalidFilename(String),
    InvalidPng,
    UnsupportedDimensions { width: u32, height: u32 },
}

impl fmt::Display for ArtworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFilename(filename) => write!(
                formatter,
                "artwork filename must be a snake_case PNG filename, got {filename:?}"
            ),
            Self::InvalidPng => formatter.write_str("artwork must be a valid PNG"),
            Self::UnsupportedDimensions { width, height } => write!(
                formatter,
                "artwork dimensions must be a positive integer multiple of 84x102, got {width}x{height}"
            ),
        }
    }
}

pub fn is_valid_artwork_filename(filename: &str) -> bool {
    let Some(stem) = filename.strip_suffix(".png") else {
        return false;
    };

    !stem.is_empty()
        && stem.split('_').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

pub fn validate_custom_artwork(bytes: &[u8]) -> Result<(u32, u32), ArtworkError> {
    let (width, height) = super::png_dimensions(bytes).ok_or(ArtworkError::InvalidPng)?;
    let width_scale = width / CARD_ARTWORK_WIDTH;
    let height_scale = height / CARD_ARTWORK_HEIGHT;
    if width_scale == 0
        || width % CARD_ARTWORK_WIDTH != 0
        || height % CARD_ARTWORK_HEIGHT != 0
        || width_scale != height_scale
    {
        return Err(ArtworkError::UnsupportedDimensions { width, height });
    }

    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes
    }

    #[test]
    fn resolves_catalog_filenames_under_the_configured_base_url() {
        let resolver = ArtworkResolver::new("/custom/", "/fallback.png");
        let sources = resolver.resolve("lizard_man.png").unwrap();
        assert_eq!(sources.primary_url, "/custom/lizard_man.png");
        assert_eq!(sources.fallback_url, "/fallback.png");
    }

    #[test]
    fn accepts_only_snake_case_png_filenames() {
        for filename in ["goblin.png", "lizard_man.png", "excalibur_ii.png"] {
            assert!(is_valid_artwork_filename(filename), "rejected {filename}");
        }
        for filename in [
            "",
            ".png",
            "Goblin.png",
            "lizard man.png",
            "lizard__man.png",
            "../goblin.png",
            "goblin.webp",
            "goblin.png?version=1",
        ] {
            assert!(!is_valid_artwork_filename(filename), "accepted {filename}");
        }
    }

    #[test]
    fn accepts_integer_multiples_of_the_canonical_dimensions() {
        for scale in [1, 2, 4] {
            let width = CARD_ARTWORK_WIDTH * scale;
            let height = CARD_ARTWORK_HEIGHT * scale;
            assert_eq!(
                validate_custom_artwork(&png_header(width, height)),
                Ok((width, height))
            );
        }
    }

    #[test]
    fn rejects_invalid_pngs_and_unsupported_dimensions() {
        assert_eq!(
            validate_custom_artwork(b"not a png"),
            Err(ArtworkError::InvalidPng)
        );
        assert_eq!(
            validate_custom_artwork(&png_header(168, 102)),
            Err(ArtworkError::UnsupportedDimensions {
                width: 168,
                height: 102,
            })
        );
    }

    #[test]
    fn load_failure_switches_to_the_fallback_permanently() {
        let sources = ArtworkResolver::new("/custom", "/fallback.png")
            .resolve("goblin.png")
            .unwrap();
        let mut state = ArtworkLoadState::Primary;
        assert_eq!(state.source(&sources), "/custom/goblin.png");

        state.use_fallback();
        assert_eq!(state.source(&sources), "/fallback.png");
        state.use_fallback();
        assert_eq!(state.source(&sources), "/fallback.png");
    }

    #[test]
    fn supplied_custom_artwork_files_are_valid() {
        let catalog_filenames = super::super::CARDS
            .iter()
            .map(|card| card.artwork.filename.as_str())
            .collect::<std::collections::HashSet<_>>();
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/cards/custom");
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("png") {
                continue;
            }

            let filename = path.file_name().unwrap().to_str().unwrap();
            assert!(
                is_valid_artwork_filename(filename),
                "{}: invalid artwork filename",
                path.display()
            );
            assert!(
                catalog_filenames.contains(filename),
                "{}: filename is not declared in the catalog",
                path.display()
            );
            let bytes = std::fs::read(&path).unwrap();
            validate_custom_artwork(&bytes)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }
}
