use super::zip::ZipError;
use std::fmt;

#[derive(Debug)]
pub enum ReaderError {
    /// ZIP archive could not be opened or is malformed.
    Zip(ZipError),
    /// manifest.json not found in the archive.
    ManifestNotFound,
    /// No animations listed in the manifest.
    NoAnimations,
    /// Requested animation not found in the archive.
    AnimationNotFound,
    /// Requested file not found in the archive.
    FileNotFound,
    /// Content is not valid UTF-8.
    InvalidUtf8(std::str::Utf8Error),
    /// JSON parsing failed.
    InvalidJson(serde_json::Error),
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip(e) => write!(f, "ZIP error: {e}"),
            Self::ManifestNotFound => f.write_str("manifest.json not found"),
            Self::NoAnimations => f.write_str("no animations listed in manifest"),
            Self::AnimationNotFound => f.write_str("animation not found in archive"),
            Self::FileNotFound => f.write_str("file not found in archive"),
            Self::InvalidUtf8(e) => write!(f, "invalid UTF-8: {e}"),
            Self::InvalidJson(e) => write!(f, "invalid JSON: {e}"),
        }
    }
}

impl std::error::Error for ReaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Zip(e) => Some(e),
            Self::InvalidUtf8(e) => Some(e),
            Self::InvalidJson(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::str::Utf8Error> for ReaderError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::InvalidUtf8(e)
    }
}

impl From<serde_json::Error> for ReaderError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidJson(e)
    }
}
