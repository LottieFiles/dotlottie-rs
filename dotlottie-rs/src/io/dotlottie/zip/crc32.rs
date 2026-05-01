//! CRC32 (IEEE/zlib polynomial `0xedb88320`) for ZIP entry integrity checks.
//!
//! Table-based byte-at-a-time Sarwate algorithm. The 1 KiB lookup table is
//! built at compile time, so this adds no startup cost.

const fn make_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut c = i;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[i as usize] = c;
        i += 1;
    }
    table
}

const TABLE: [u32; 256] = make_table();

#[inline]
pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc = TABLE[((crc ^ b as u32) & 0xff) as usize] ^ (crc >> 8);
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn known_vectors() {
        assert_eq!(crc32(b"a"), 0xe8b7be43);
        assert_eq!(crc32(b"abc"), 0x352441c2);
        assert_eq!(
            crc32(b"The quick brown fox jumps over the lazy dog"),
            0x414fa339
        );
    }
}
