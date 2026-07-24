//! Minimal read-only ZIP reader for the dotLottie container: central
//! directory walk + `miniz_oxide` inflate, stored/deflate methods only.
//! ZIP64, encryption, and multi-disk archives are rejected. CRC32 is not
//! validated: corrupt entries fail downstream at JSON/image/audio decode.

use std::borrow::Cow;
use std::ops::Range;

const EOCD_SIG: u32 = 0x0605_4b50;
const CDIR_SIG: u32 = 0x0201_4b50;
const LOCAL_SIG: u32 = 0x0403_4b50;

const EOCD_MIN: usize = 22;
const MAX_COMMENT: usize = 0xFFFF;

const METHOD_STORED: u16 = 0;
const METHOD_DEFLATE: u16 = 8;

const MAX_ENTRIES: usize = 4096;
const MAX_ENTRY_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArchiveError {
    Invalid,
    TooLarge,
    Unsupported,
    NotFound,
    Inflate,
}

#[derive(Debug)]
pub(crate) struct Archive {
    data: Vec<u8>,
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    name: Range<usize>,
    method: u16,
    data: Range<usize>,
    uncompressed_size: usize,
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    let b = data.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    let b = data.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Bounds-checked `start..start + len`; `checked_add` guards usize wrap on
/// 32-bit targets with crafted header values.
fn range_at(data: &[u8], start: usize, len: usize) -> Option<Range<usize>> {
    let end = start.checked_add(len)?;
    data.get(start..end)?;
    Some(start..end)
}

impl Archive {
    pub fn new(data: Vec<u8>) -> Result<Self, ArchiveError> {
        let eocd = Self::find_eocd(&data).ok_or(ArchiveError::Invalid)?;
        let disk_number = u16_at(&data, eocd + 4).ok_or(ArchiveError::Invalid)?;
        let cdir_disk = u16_at(&data, eocd + 6).ok_or(ArchiveError::Invalid)?;
        if disk_number != 0 || cdir_disk != 0 {
            return Err(ArchiveError::Unsupported);
        }
        let entry_count = u16_at(&data, eocd + 10).ok_or(ArchiveError::Invalid)? as usize;
        let cdir_offset = u32_at(&data, eocd + 16).ok_or(ArchiveError::Invalid)? as usize;
        if entry_count == 0xFFFF || cdir_offset == 0xFFFF_FFFF {
            return Err(ArchiveError::Unsupported); // ZIP64
        }
        if entry_count > MAX_ENTRIES {
            return Err(ArchiveError::TooLarge);
        }

        let mut entries = Vec::with_capacity(entry_count);
        let mut at = cdir_offset;
        for _ in 0..entry_count {
            if u32_at(&data, at) != Some(CDIR_SIG) {
                return Err(ArchiveError::Invalid);
            }
            let flags = u16_at(&data, at + 8).ok_or(ArchiveError::Invalid)?;
            let method = u16_at(&data, at + 10).ok_or(ArchiveError::Invalid)?;
            let compressed_size = u32_at(&data, at + 20).ok_or(ArchiveError::Invalid)? as usize;
            let uncompressed_size = u32_at(&data, at + 24).ok_or(ArchiveError::Invalid)? as usize;
            let name_len = u16_at(&data, at + 28).ok_or(ArchiveError::Invalid)? as usize;
            let extra_len = u16_at(&data, at + 30).ok_or(ArchiveError::Invalid)? as usize;
            let comment_len = u16_at(&data, at + 32).ok_or(ArchiveError::Invalid)? as usize;
            let local_offset = u32_at(&data, at + 42).ok_or(ArchiveError::Invalid)? as usize;
            if compressed_size == 0xFFFF_FFFF
                || uncompressed_size == 0xFFFF_FFFF
                || local_offset == 0xFFFF_FFFF
            {
                return Err(ArchiveError::Unsupported); // ZIP64
            }
            let name = range_at(&data, at + 46, name_len).ok_or(ArchiveError::Invalid)?;
            at = name
                .end
                .checked_add(extra_len)
                .and_then(|v| v.checked_add(comment_len))
                .ok_or(ArchiveError::Invalid)?;

            let name_bytes = &data[name.clone()];
            // Directory and non-UTF-8 names are unreferenceable from manifest.json.
            if name_bytes.ends_with(b"/") || std::str::from_utf8(name_bytes).is_err() {
                continue;
            }
            if flags & 0x1 != 0 {
                return Err(ArchiveError::Unsupported); // encrypted
            }
            if method != METHOD_STORED && method != METHOD_DEFLATE {
                return Err(ArchiveError::Unsupported);
            }
            if uncompressed_size > MAX_ENTRY_SIZE {
                return Err(ArchiveError::TooLarge);
            }

            // Name/extra lengths may differ from the central directory's copy;
            // the payload offset must come from the local header.
            if u32_at(&data, local_offset) != Some(LOCAL_SIG) {
                return Err(ArchiveError::Invalid);
            }
            let local_name_len =
                u16_at(&data, local_offset + 26).ok_or(ArchiveError::Invalid)? as usize;
            let local_extra_len =
                u16_at(&data, local_offset + 28).ok_or(ArchiveError::Invalid)? as usize;
            let payload_start = local_offset
                .checked_add(30 + local_name_len)
                .and_then(|v| v.checked_add(local_extra_len))
                .ok_or(ArchiveError::Invalid)?;
            let payload =
                range_at(&data, payload_start, compressed_size).ok_or(ArchiveError::Invalid)?;

            entries.push(Entry {
                name,
                method,
                data: payload,
                uncompressed_size,
            });
        }

        Ok(Archive { data, entries })
    }

    fn find_eocd(data: &[u8]) -> Option<usize> {
        let start = data.len().checked_sub(EOCD_MIN)?;
        let floor = start.saturating_sub(MAX_COMMENT);
        (floor..=start)
            .rev()
            .find(|&at| u32_at(data, at) == Some(EOCD_SIG))
    }

    /// Stored entries borrow from the archive buffer; deflated entries
    /// inflate into a fresh buffer capped at the declared size.
    pub fn by_name(&self, name: &str) -> Result<Cow<'_, [u8]>, ArchiveError> {
        let entry = self
            .entries
            .iter()
            .find(|e| &self.data[e.name.clone()] == name.as_bytes())
            .ok_or(ArchiveError::NotFound)?;
        let raw = &self.data[entry.data.clone()];
        match entry.method {
            METHOD_STORED => Ok(Cow::Borrowed(raw)),
            _ => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, entry.uncompressed_size)
                .map(Cow::Owned)
                .map_err(|_| ArchiveError::Inflate),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    //! Tiny ZIP writer for tests; CRC fields are zeroed (the reader skips them).

    /// Entries are `(name, content, deflate?)`.
    pub fn build_zip(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, content, deflate) in entries {
            let payload = if *deflate {
                miniz_oxide::deflate::compress_to_vec(content, 6)
            } else {
                content.to_vec()
            };
            let method: u16 = if *deflate { 8 } else { 0 };
            let offset = out.len() as u32;
            // Local file header (30 bytes + name).
            out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
            out.extend_from_slice(&20u16.to_le_bytes()); // version needed
            out.extend_from_slice(&0u16.to_le_bytes()); // flags
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&[0u8; 4]); // dos time/date
            out.extend_from_slice(&[0u8; 4]); // crc32
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&(content.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes()); // extra len
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&payload);
            // Central directory record (46 bytes + name).
            central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
            central.extend_from_slice(&20u16.to_le_bytes()); // version made by
            central.extend_from_slice(&20u16.to_le_bytes()); // version needed
            central.extend_from_slice(&0u16.to_le_bytes()); // flags
            central.extend_from_slice(&method.to_le_bytes());
            central.extend_from_slice(&[0u8; 4]); // dos time/date
            central.extend_from_slice(&[0u8; 4]); // crc32
            central.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            central.extend_from_slice(&(content.len() as u32).to_le_bytes());
            central.extend_from_slice(&(name.len() as u16).to_le_bytes());
            // extra len, comment len, disk#, internal attrs, external attrs
            central.extend_from_slice(&[0u8; 12]);
            central.extend_from_slice(&offset.to_le_bytes());
            central.extend_from_slice(name.as_bytes());
        }
        let cdir_offset = out.len() as u32;
        let cdir_size = central.len() as u32;
        out.extend_from_slice(&central);
        // End of central directory (22 bytes).
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&[0u8; 4]); // disk numbers
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&cdir_size.to_le_bytes());
        out.extend_from_slice(&cdir_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // comment len
        out
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::build_zip;
    use super::*;

    #[test]
    fn roundtrips_stored_entries() {
        let zip = build_zip(&[
            ("manifest.json", b"{}", false),
            ("a/x.json", b"hello", false),
        ]);
        let archive = Archive::new(zip).unwrap();
        assert_eq!(archive.by_name("manifest.json").unwrap().as_ref(), b"{}");
        assert_eq!(archive.by_name("a/x.json").unwrap().as_ref(), b"hello");
        assert!(matches!(
            archive.by_name("a/x.json").unwrap(),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn roundtrips_deflated_entries() {
        let payload = b"the quick brown fox jumps over the lazy dog".repeat(50);
        let zip = build_zip(&[("a/big.json", &payload, true)]);
        let archive = Archive::new(zip).unwrap();
        assert_eq!(
            archive.by_name("a/big.json").unwrap().as_ref(),
            &payload[..]
        );
    }

    #[test]
    fn roundtrips_empty_entry() {
        let zip = build_zip(&[("empty.json", b"", false)]);
        let archive = Archive::new(zip).unwrap();
        assert_eq!(archive.by_name("empty.json").unwrap().len(), 0);
    }

    #[test]
    fn missing_name_is_not_found() {
        let zip = build_zip(&[("manifest.json", b"{}", false)]);
        let archive = Archive::new(zip).unwrap();
        assert_eq!(
            archive.by_name("nope.json").unwrap_err(),
            ArchiveError::NotFound
        );
    }

    #[test]
    fn reads_real_fixture() {
        let path = format!(
            "{}/assets/animations/dotlottie/v1/emojis.lottie",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(path).expect("fixture must exist");
        let archive = Archive::new(bytes).expect("fixture parses");
        let manifest = archive.by_name("manifest.json").expect("manifest entry");
        assert!(std::str::from_utf8(&manifest)
            .unwrap()
            .contains("animations"));
    }

    #[test]
    fn rejects_non_zip_input() {
        assert_eq!(
            Archive::new(b"{\"v\":\"5.5\"}".to_vec()).unwrap_err(),
            ArchiveError::Invalid
        );
        assert_eq!(Archive::new(Vec::new()).unwrap_err(), ArchiveError::Invalid);
    }

    #[test]
    fn rejects_truncated_archive() {
        let zip = build_zip(&[("a.json", b"data", false)]);
        for len in [zip.len() - 1, zip.len() - 10, 5] {
            assert!(Archive::new(zip[..len].to_vec()).is_err(), "len {len}");
        }
    }

    #[test]
    fn parses_with_trailing_comment() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let comment = b"trailing comment";
        let n = zip.len();
        zip[n - 2..].copy_from_slice(&(comment.len() as u16).to_le_bytes());
        zip.extend_from_slice(comment);
        let archive = Archive::new(zip).unwrap();
        assert_eq!(archive.by_name("a.json").unwrap().as_ref(), b"data");
    }

    #[test]
    fn rejects_zip64_entry_count_sentinel() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let eocd = zip.len() - 22;
        zip[eocd + 10..eocd + 12].copy_from_slice(&0xFFFFu16.to_le_bytes());
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::Unsupported);
    }

    #[test]
    fn rejects_unsupported_method() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let cd = zip
            .windows(4)
            .position(|w| w == 0x0201_4b50u32.to_le_bytes())
            .unwrap();
        zip[cd + 10..cd + 12].copy_from_slice(&12u16.to_le_bytes()); // bzip2
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::Unsupported);
    }

    #[test]
    fn skips_directory_and_non_utf8_entries() {
        let zip = build_zip(&[
            ("dir/", b"", false),
            ("\u{fffd}ok.json", b"x", false),
            ("real.json", b"y", false),
        ]);
        // Corrupt the second entry's name to invalid UTF-8 in both headers.
        let mut zip = zip;
        let needle = "\u{fffd}".as_bytes().to_vec();
        while let Some(i) = zip.windows(needle.len()).position(|w| w == needle) {
            zip[i..i + needle.len()].copy_from_slice(&[0xFF, 0xFE, 0xFD]);
        }
        let archive = Archive::new(zip).unwrap();
        assert_eq!(archive.by_name("real.json").unwrap().as_ref(), b"y");
        assert_eq!(archive.by_name("dir/").unwrap_err(), ArchiveError::NotFound);
    }

    #[test]
    fn rejects_entry_count_over_cap() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let eocd = zip.len() - 22;
        zip[eocd + 10..eocd + 12].copy_from_slice(&4097u16.to_le_bytes());
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::TooLarge);
    }

    #[test]
    fn rejects_oversized_declared_entry() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let cd = zip
            .windows(4)
            .position(|w| w == 0x0201_4b50u32.to_le_bytes())
            .unwrap();
        // uncompressed_size field at CD offset +24.
        zip[cd + 24..cd + 28].copy_from_slice(&(65 * 1024 * 1024u32).to_le_bytes());
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::TooLarge);
    }

    #[test]
    fn inflate_output_capped_at_declared_size() {
        let payload = b"A".repeat(10_000);
        let mut zip = build_zip(&[("a.bin", &payload, true)]);
        let cd = zip
            .windows(4)
            .position(|w| w == 0x0201_4b50u32.to_le_bytes())
            .unwrap();
        zip[cd + 24..cd + 28].copy_from_slice(&16u32.to_le_bytes());
        let archive = Archive::new(zip).unwrap();
        assert_eq!(archive.by_name("a.bin").unwrap_err(), ArchiveError::Inflate);
    }

    #[test]
    fn rejects_wrapping_local_offset() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let cd = zip
            .windows(4)
            .position(|w| w == 0x0201_4b50u32.to_le_bytes())
            .unwrap();
        // local_header_offset field at CD offset +42: huge but non-sentinel.
        zip[cd + 42..cd + 46].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::Invalid);
    }

    #[test]
    fn respects_local_header_extra_field() {
        // Local header carries a 4-byte extra field the central directory omits.
        let name = b"a.json";
        let content = b"data";
        let mut out = Vec::new();
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&[0u8; 8]); // time/date + crc
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&4u16.to_le_bytes()); // local extra len = 4
        out.extend_from_slice(name);
        out.extend_from_slice(&[0xAA; 4]); // the extra field
        out.extend_from_slice(content);
        let cdir_offset = out.len() as u32;
        out.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&[0u8; 8]);
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&[0u8; 12]); // CD extra len = 0
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(name);
        let cdir_size = out.len() as u32 - cdir_offset;
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&[0u8; 4]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&cdir_size.to_le_bytes());
        out.extend_from_slice(&cdir_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());

        let archive = Archive::new(out).unwrap();
        assert_eq!(archive.by_name("a.json").unwrap().as_ref(), b"data");
    }

    #[test]
    fn rejects_encrypted_entry() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let cd = zip
            .windows(4)
            .position(|w| w == 0x0201_4b50u32.to_le_bytes())
            .unwrap();
        // flags field at CD offset +8: set bit 0 (encrypted).
        zip[cd + 8..cd + 10].copy_from_slice(&1u16.to_le_bytes());
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::Unsupported);
    }

    #[test]
    fn rejects_multi_disk_archive() {
        let mut zip = build_zip(&[("a.json", b"data", false)]);
        let eocd = zip.len() - 22;
        zip[eocd + 4..eocd + 6].copy_from_slice(&1u16.to_le_bytes());
        assert_eq!(Archive::new(zip).unwrap_err(), ArchiveError::Unsupported);
    }

    #[test]
    fn duplicate_names_first_wins() {
        let zip = build_zip(&[("a.json", b"first", false), ("a.json", b"second", false)]);
        let archive = Archive::new(zip).unwrap();
        assert_eq!(archive.by_name("a.json").unwrap().as_ref(), b"first");
    }
}
