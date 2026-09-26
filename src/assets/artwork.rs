use std::fmt;

pub const CARD_ARTWORK_COUNT: usize = 100;
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

    pub fn resolve(self, definition_index: usize) -> Result<ArtworkSources, ArtworkError> {
        if definition_index >= CARD_ARTWORK_COUNT {
            return Err(ArtworkError::DefinitionIndexOutOfRange(definition_index));
        }

        Ok(ArtworkSources {
            primary_url: format!(
                "{}/Card{:03}.png",
                self.custom_base_url,
                definition_index + 1
            ),
            fallback_url: self.fallback_url.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtworkError {
    DefinitionIndexOutOfRange(usize),
    InvalidFilename(String),
    InvalidPng,
    UnsupportedDimensions { width: u32, height: u32 },
}

impl fmt::Display for ArtworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefinitionIndexOutOfRange(index) => {
                write!(formatter, "card definition index {index} is out of range")
            }
            Self::InvalidFilename(filename) => write!(
                formatter,
                "artwork filename must be Card001.png through Card100.png, got {filename:?}"
            ),
            Self::InvalidPng => formatter.write_str("artwork must be a valid PNG"),
            Self::UnsupportedDimensions { width, height } => write!(
                formatter,
                "artwork dimensions must be a positive integer multiple of 84x102, got {width}x{height}"
            ),
        }
    }
}

pub fn definition_index_from_filename(filename: &str) -> Result<usize, ArtworkError> {
    let number = filename
        .strip_prefix("Card")
        .and_then(|value| value.strip_suffix(".png"))
        .filter(|value| value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| ArtworkError::InvalidFilename(filename.to_owned()))?;
    let definition_index = number
        .checked_sub(1)
        .ok_or_else(|| ArtworkError::InvalidFilename(filename.to_owned()))?;
    if definition_index >= CARD_ARTWORK_COUNT {
        return Err(ArtworkError::InvalidFilename(filename.to_owned()));
    }

    Ok(definition_index)
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
    fn resolves_every_catalog_index_to_its_one_based_filename() {
        let resolver = ArtworkResolver::new("/custom/", "/fallback.png");

        for index in 0..CARD_ARTWORK_COUNT {
            let sources = resolver.resolve(index).unwrap();
            assert_eq!(
                sources.primary_url,
                format!("/custom/Card{:03}.png", index + 1)
            );
            assert_eq!(sources.fallback_url, "/fallback.png");
        }
    }

    #[test]
    fn rejects_definition_indices_outside_the_catalog() {
        let error = ArtworkResolver::new("/custom", "/fallback.png")
            .resolve(CARD_ARTWORK_COUNT)
            .unwrap_err();

        assert_eq!(
            error,
            ArtworkError::DefinitionIndexOutOfRange(CARD_ARTWORK_COUNT)
        );
    }

    #[test]
    fn accepts_only_canonical_custom_artwork_filenames() {
        assert_eq!(definition_index_from_filename("Card001.png"), Ok(0));
        assert_eq!(definition_index_from_filename("Card100.png"), Ok(99));
        for filename in [
            "Card000.png",
            "Card101.png",
            "Card1.png",
            "card001.png",
            "Card001.webp",
        ] {
            assert!(
                matches!(
                    definition_index_from_filename(filename),
                    Err(ArtworkError::InvalidFilename(_))
                ),
                "accepted {filename}"
            );
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
            .resolve(0)
            .unwrap();
        let mut state = ArtworkLoadState::Primary;
        assert_eq!(state.source(&sources), "/custom/Card001.png");

        state.use_fallback();
        assert_eq!(state.source(&sources), "/fallback.png");
        state.use_fallback();
        assert_eq!(state.source(&sources), "/fallback.png");
    }

    #[test]
    fn supplied_custom_artwork_files_are_valid() {
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/cards/custom");
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("png") {
                continue;
            }

            let filename = path.file_name().unwrap().to_str().unwrap();
            definition_index_from_filename(filename)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let bytes = std::fs::read(&path).unwrap();
            validate_custom_artwork(&bytes)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }
}
