mod crc32;
mod decompress;
mod error;
mod inflate;
mod parse;

pub use error::ZipError;
pub use inflate::InflateError;

use parse::CentralDirEntry;
use std::borrow::Cow;
use std::fmt;

pub(crate) struct Archive {
    data: Vec<u8>,
    entries: Vec<CentralDirEntry>,
}

impl fmt::Debug for Archive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Archive")
            .field("entries", &self.entries.len())
            .finish()
    }
}

impl Archive {
    pub fn new(data: Vec<u8>) -> Result<Self, ZipError> {
        let entries = parse::parse_archive(&data)?;
        Ok(Self { data, entries })
    }

    pub fn read(&self, name: &str) -> Result<Cow<'_, [u8]>, ZipError> {
        let idx = self
            .entries
            .binary_search_by(|e| e.name.as_ref().cmp(name))
            .map_err(|_| ZipError::FileNotFound)?;
        let entry = &self.entries[idx];
        let (offset, len) = parse::locate_file_data(&self.data, entry)?;
        decompress::decompress(
            &self.data[offset..offset + len],
            entry.compression_method,
            entry.uncompressed_size as usize,
            entry.crc32,
        )
    }
}
