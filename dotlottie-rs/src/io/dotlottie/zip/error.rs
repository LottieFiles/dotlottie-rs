use super::inflate::InflateError;
use std::fmt;

#[derive(Debug)]
pub enum ZipError {
    /// Buffer is smaller than the minimum-size end-of-central-directory record.
    TooSmall,
    /// EOCD signature not found in the trailing 64 KiB of the buffer.
    EocdNotFound,
    /// Central directory header is malformed or out of bounds.
    InvalidCentralDir,
    /// Local file header is malformed or out of bounds.
    InvalidLocalHeader,
    /// Compression method is neither STORE (0) nor DEFLATE (8).
    UnsupportedCompression(u16),
    /// Raw DEFLATE inflate failed.
    Decompress(InflateError),
    /// Decompressed length doesn't match the central-directory entry.
    UncompressedSizeMismatch { expected: usize, actual: usize },
    /// CRC32 of the decompressed payload doesn't match the entry.
    ChecksumMismatch { expected: u32, actual: u32 },
    /// Entry's uncompressed size exceeds the per-file limit.
    EntryTooLarge { uncompressed_size: usize },
    /// Requested entry name not present in the archive.
    FileNotFound,
    /// A length-prefixed read would extend past the end of the buffer.
    OutOfBounds,
}

impl fmt::Display for ZipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall => f.write_str("archive smaller than minimum ZIP size"),
            Self::EocdNotFound => f.write_str("end-of-central-directory record not found"),
            Self::InvalidCentralDir => f.write_str("malformed central directory"),
            Self::InvalidLocalHeader => f.write_str("malformed local file header"),
            Self::UnsupportedCompression(method) => {
                write!(f, "unsupported compression method {method}")
            }
            Self::Decompress(e) => write!(f, "decompression failed: {e}"),
            Self::UncompressedSizeMismatch { expected, actual } => write!(
                f,
                "uncompressed size mismatch: expected {expected} bytes, got {actual}",
            ),
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "CRC32 mismatch: expected {expected:#010x}, got {actual:#010x}",
            ),
            Self::EntryTooLarge { uncompressed_size } => write!(
                f,
                "entry uncompressed size {uncompressed_size} exceeds per-file limit",
            ),
            Self::FileNotFound => f.write_str("file not found in archive"),
            Self::OutOfBounds => f.write_str("read out of bounds"),
        }
    }
}

impl std::error::Error for ZipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decompress(e) => Some(e),
            _ => None,
        }
    }
}
