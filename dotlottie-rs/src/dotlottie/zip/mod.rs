mod decompress;
mod parse;

pub use parse::ZipError;

use parse::CentralDirEntry;
use std::borrow::Cow;
use std::cell::Cell;
use std::fmt;

struct DecompressState {
    decompressor: libdeflater::Decompressor,
    buf: Vec<u8>,
}

pub(crate) struct DotLottieArchive {
    data: Vec<u8>,
    entries: Vec<CentralDirEntry>,
    decompress_state: Cell<Option<DecompressState>>,
}

impl fmt::Debug for DotLottieArchive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DotLottieArchive")
            .field("entries", &self.entries.len())
            .finish()
    }
}

impl DotLottieArchive {
    pub fn new(data: Vec<u8>) -> Result<Self, ZipError> {
        let entries = parse::parse_archive(&data)?;
        Ok(Self {
            data,
            entries,
            decompress_state: Cell::new(None),
        })
    }

    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.name.as_ref())
    }

    pub fn read(&self, name: &str) -> Result<Cow<'_, [u8]>, ZipError> {
        let idx = self
            .entries
            .binary_search_by(|e| e.name.as_ref().cmp(name))
            .map_err(|_| ZipError::FileNotFound)?;
        let entry = &self.entries[idx];
        let (offset, len) = parse::locate_file_data(&self.data, entry)?;
        let mut state = self
            .decompress_state
            .take()
            .unwrap_or_else(|| DecompressState {
                decompressor: libdeflater::Decompressor::new(),
                buf: Vec::new(),
            });
        let result = decompress::decompress(
            &self.data[offset..offset + len],
            entry.compression_method,
            entry.uncompressed_size as usize,
            entry.crc32,
            &mut state.decompressor,
            &mut state.buf,
        );
        self.decompress_state.set(Some(state));
        result
    }
}
