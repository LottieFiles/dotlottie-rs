mod crc32;
mod decompress;
mod error;
mod inflate;
mod parse;

pub use error::ZipError;
pub use inflate::InflateError;

use parse::CentralDirEntry;
use std::borrow::Cow;
use std::cmp::Ordering;
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
        let target = name.as_bytes();
        let idx = self
            .entries
            .binary_search_by(|e| Ord::cmp(parse::name_bytes(&self.data, e), target))
            .map_err(|_| ZipError::FileNotFound)?;
        self.read_entry(&self.entries[idx])
    }

    /// Look up an entry whose name is the logical concatenation of `prefix`
    /// and `suffix`, comparing byte-wise without allocating a joined string.
    pub fn read_concat(&self, prefix: &str, suffix: &str) -> Result<Cow<'_, [u8]>, ZipError> {
        let prefix = prefix.as_bytes();
        let suffix = suffix.as_bytes();
        let idx = self
            .entries
            .binary_search_by(|e| cmp_concat(parse::name_bytes(&self.data, e), prefix, suffix))
            .map_err(|_| ZipError::FileNotFound)?;
        self.read_entry(&self.entries[idx])
    }

    fn read_entry(&self, entry: &parse::CentralDirEntry) -> Result<Cow<'_, [u8]>, ZipError> {
        let (offset, len) = parse::locate_file_data(&self.data, entry)?;
        decompress::decompress(
            &self.data[offset..offset + len],
            entry.compression_method,
            entry.uncompressed_size as usize,
            entry.crc32,
        )
    }

    /// Returns `true` when the archive contains exactly one entry under
    /// `animation_prefix` and zero entries under `t/` (themes) or `s/`
    /// (state machines). Other entries (assets under `i/`, `u/`, `f/`,
    /// `images/`, `audio/`, plus `manifest.json`) are ignored — once
    /// `Animation::load_data` has resolved external assets eagerly, those
    /// entries serve no further purpose to the player.
    ///
    /// In that case the caller can drop the archive after a successful
    /// load, since none of the reader-backed APIs (`apply_theme`,
    /// `load_state_machine`, `load_animation(other)`) could ever do
    /// anything useful with it.
    pub fn is_single_shot(&self, animation_prefix: &[u8]) -> bool {
        let mut animation_count: usize = 0;
        for entry in &self.entries {
            let name = parse::name_bytes(&self.data, entry);
            if name.starts_with(b"t/") || name.starts_with(b"s/") {
                return false;
            }
            if name.starts_with(animation_prefix) {
                animation_count += 1;
                if animation_count > 1 {
                    return false;
                }
            }
        }
        animation_count == 1
    }
}

/// Compare `name` against the logical concatenation `prefix + suffix` without
/// materialising the joined string.
#[inline]
fn cmp_concat(name: &[u8], prefix: &[u8], suffix: &[u8]) -> Ordering {
    let p = prefix.len();
    if name.len() < p {
        match name.cmp(&prefix[..name.len()]) {
            Ordering::Equal => Ordering::Less,
            other => other,
        }
    } else {
        match name[..p].cmp(prefix) {
            Ordering::Equal => name[p..].cmp(suffix),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::cmp_concat;
    use std::cmp::Ordering;

    #[test]
    fn cmp_concat_full_match() {
        assert_eq!(
            cmp_concat(b"images/foo.png", b"images/", b"foo.png"),
            Ordering::Equal
        );
    }

    #[test]
    fn cmp_concat_name_shorter_than_prefix_matching() {
        // name is a strict byte-prefix of `prefix` → name sorts before prefix+suffix.
        assert_eq!(cmp_concat(b"image", b"images/", b"foo"), Ordering::Less);
        assert_eq!(cmp_concat(b"images", b"images/", b"foo"), Ordering::Less);
    }

    #[test]
    fn cmp_concat_name_shorter_diverges() {
        // first divergent byte governs the ordering.
        assert_eq!(cmp_concat(b"j", b"images/", b"foo"), Ordering::Greater);
        assert_eq!(cmp_concat(b"a", b"images/", b"foo"), Ordering::Less);
    }

    #[test]
    fn cmp_concat_prefix_diverges() {
        assert_eq!(
            cmp_concat(b"jmages/foo", b"images/", b"foo"),
            Ordering::Greater
        );
        assert_eq!(
            cmp_concat(b"hmages/foo", b"images/", b"foo"),
            Ordering::Less
        );
    }

    #[test]
    fn cmp_concat_suffix_governs() {
        assert_eq!(
            cmp_concat(b"images/zzz", b"images/", b"foo"),
            Ordering::Greater
        );
        assert_eq!(
            cmp_concat(b"images/aaa", b"images/", b"foo"),
            Ordering::Less
        );
        assert_eq!(
            cmp_concat(b"images/foo.png", b"images/", b"foo.png"),
            Ordering::Equal
        );
    }

    #[test]
    fn cmp_concat_name_longer_than_prefix_only() {
        // name = prefix + something, suffix mismatch decides.
        assert_eq!(
            cmp_concat(b"images/abc", b"images/", b""),
            Ordering::Greater
        );
        assert_eq!(cmp_concat(b"images/", b"images/", b""), Ordering::Equal);
    }

    #[test]
    fn cmp_concat_empty_prefix() {
        assert_eq!(cmp_concat(b"foo", b"", b"foo"), Ordering::Equal);
        assert_eq!(cmp_concat(b"foo", b"", b"bar"), Ordering::Greater);
    }
}
