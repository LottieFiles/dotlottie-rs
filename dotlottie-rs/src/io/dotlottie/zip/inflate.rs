//! Minimal raw DEFLATE (RFC 1951) inflater optimized for dotLottie archives.
//!
//! Supports all three block types: stored (0), fixed Huffman (1), dynamic Huffman (2).
//! Uses two-level lookup tables for fast Huffman decoding.

use std::fmt;
use std::sync::OnceLock;

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
#[non_exhaustive]
pub enum InflateError {
    /// Reserved (BTYPE=3) block encountered.
    InvalidBlockType,
    /// Code-length code stream produced lengths that don't form a valid
    /// canonical Huffman table.
    InvalidHuffmanTable,
    /// Stored block's LEN doesn't match `!NLEN`.
    InvalidStoredLen,
    /// Decoded literal/length symbol is outside the 257..=285 range.
    InvalidLengthCode,
    /// Decoded distance symbol is outside the 0..=29 range.
    InvalidDistanceCode,
    /// Back-reference distance exceeds the bytes already produced.
    DistanceTooFarBack,
    /// Input ended mid-block.
    UnexpectedEof,
    /// Decoder produced more bytes than the caller-supplied limit.
    /// Indicates a malformed DEFLATE stream (or one that disagrees with the
    /// declared uncompressed size); aborts before unbounded `Vec` growth.
    OutputTooLarge,
}

impl fmt::Display for InflateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::InvalidBlockType => "invalid DEFLATE block type",
            Self::InvalidHuffmanTable => "malformed Huffman table",
            Self::InvalidStoredLen => "stored block LEN/NLEN mismatch",
            Self::InvalidLengthCode => "invalid length code",
            Self::InvalidDistanceCode => "invalid distance code",
            Self::DistanceTooFarBack => "back-reference distance exceeds output",
            Self::UnexpectedEof => "unexpected end of input",
            Self::OutputTooLarge => "decoder produced more output than the declared size",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for InflateError {}

// ── Static tables (RFC 1951 §3.2.5) ────────────────────────────────────────

/// Length codes 257..285 → (base_length, extra_bits)
const LENGTH_TABLE: [(u16, u8); 29] = [
    (3, 0),
    (4, 0),
    (5, 0),
    (6, 0),
    (7, 0),
    (8, 0),
    (9, 0),
    (10, 0),
    (11, 1),
    (13, 1),
    (15, 1),
    (17, 1),
    (19, 2),
    (23, 2),
    (27, 2),
    (31, 2),
    (35, 3),
    (43, 3),
    (51, 3),
    (59, 3),
    (67, 4),
    (83, 4),
    (99, 4),
    (115, 4),
    (131, 5),
    (163, 5),
    (195, 5),
    (227, 5),
    (258, 0),
];

/// Distance codes 0..29 → (base_distance, extra_bits)
const DISTANCE_TABLE: [(u16, u8); 30] = [
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 1),
    (7, 1),
    (9, 2),
    (13, 2),
    (17, 3),
    (25, 3),
    (33, 4),
    (49, 4),
    (65, 5),
    (97, 5),
    (129, 6),
    (193, 6),
    (257, 7),
    (385, 7),
    (513, 8),
    (769, 8),
    (1025, 9),
    (1537, 9),
    (2049, 10),
    (3073, 10),
    (4097, 11),
    (6145, 11),
    (8193, 12),
    (12289, 12),
    (16385, 13),
    (24577, 13),
];

/// Order in which code-length code lengths are transmitted (RFC 1951 §3.2.7)
const CODELEN_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

// ── Bit reader ──────────────────────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u32,
    bits: u8,
    /// Bits currently in `buf` that came from real input bytes. The high
    /// `bits - real_bits` bits are zero-padding from past `data.len()`.
    /// `fill` loads in 8-bit chunks, so a request for 3 bits at the end of
    /// a stream may legitimately include zero-pad in the buffer — but as
    /// long as we don't *consume* those high bits, the decode is sound.
    real_bits: u8,
    /// Set when `consume` advanced past `real_bits` — i.e. some committed
    /// bit was zero-pad rather than real data. Inflater treats this as a
    /// hard signal that the stream ended mid-symbol.
    eof_consumed: bool,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            buf: 0,
            bits: 0,
            real_bits: 0,
            eof_consumed: false,
        }
    }

    /// Fill the bit buffer so it contains at least `need` bits (max 25).
    /// Past-EOF reads load zero bytes silently; the caller only learns it
    /// was zero-pad if and when those bits get `consume`d.
    #[inline]
    fn fill(&mut self, need: u8) {
        while self.bits < need {
            if self.pos < self.data.len() {
                let byte = self.data[self.pos];
                self.buf |= (byte as u32) << self.bits;
                self.real_bits += 8;
            }
            self.pos += 1;
            self.bits += 8;
        }
    }

    #[inline]
    fn peek(&mut self, n: u8) -> u16 {
        self.fill(n);
        (self.buf & ((1u32 << n) - 1)) as u16
    }

    #[inline]
    fn consume(&mut self, n: u8) {
        self.buf >>= n;
        self.bits -= n;
        if n > self.real_bits {
            self.eof_consumed = true;
            self.real_bits = 0;
        } else {
            self.real_bits -= n;
        }
    }

    #[inline]
    fn read(&mut self, n: u8) -> u16 {
        let v = self.peek(n);
        self.consume(n);
        v
    }

    /// Discard remaining bits in the current byte to align to a byte boundary.
    fn align(&mut self) {
        let discard = self.bits & 7;
        if discard > 0 {
            self.consume(discard);
        }
    }

    /// Read `n` bytes directly (after alignment). Returns a slice or error.
    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], InflateError> {
        // After alignment the bit buffer should be at a byte boundary.
        // Drain any full bytes still in the bit buffer first.
        while self.bits >= 8 {
            // Push byte back into the stream conceptually.
            self.pos -= 1;
            self.bits -= 8;
            self.buf >>= 8;
            // The byte we just pushed back was the low (oldest) byte in
            // buf. If it was real data, `real_bits` had >= 8 bits at the
            // low end; otherwise (zero-pad), real_bits was already 0.
            self.real_bits = self.real_bits.saturating_sub(8);
        }
        // Now bits == 0 after align()
        let start = self.pos;
        let end = start.checked_add(n).ok_or(InflateError::UnexpectedEof)?;
        if end > self.data.len() {
            return Err(InflateError::UnexpectedEof);
        }
        self.pos = end;
        Ok(&self.data[start..end])
    }
}

// ── Huffman table ───────────────────────────────────────────────────────────

/// Packed entry: bits [15:0] = symbol/sub-table offset, bits [19:16] = code length, bit 31 = sub-table flag
const SUB_TABLE_FLAG: u32 = 1 << 31;

#[inline]
fn entry(sym: u16, len: u8) -> u32 {
    (sym as u32) | ((len as u32) << 16)
}

#[inline]
fn entry_sym(e: u32) -> u16 {
    e as u16
}

#[inline]
fn entry_len(e: u32) -> u8 {
    (e >> 16) as u8
}

struct HuffmanTable {
    table: Vec<u32>,
    primary_bits: u8,
}

impl HuffmanTable {
    /// Build a Huffman lookup table from an array of code lengths.
    /// `lengths[i]` = code length for symbol `i` (0 means symbol is not used).
    fn build(lengths: &[u8], primary_bits: u8) -> Result<Self, InflateError> {
        let max_len = lengths.iter().copied().max().unwrap_or(0);
        if max_len == 0 {
            // All lengths zero: empty table, only valid if never queried.
            return Ok(Self {
                table: vec![0; 1 << primary_bits],
                primary_bits,
            });
        }
        if max_len > 15 {
            return Err(InflateError::InvalidHuffmanTable);
        }

        // Step 1: count codes of each length
        let mut bl_count = [0u16; 16];
        for &len in lengths {
            bl_count[len as usize] += 1;
        }
        bl_count[0] = 0;

        // Step 2: find the numerical value of the smallest code for each length
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for bits in 1..=max_len {
            code = (code + bl_count[bits as usize - 1] as u32) << 1;
            next_code[bits as usize] = code;
        }

        // Step 3: assign codes to symbols
        let mut codes = vec![(0u32, 0u8); lengths.len()];
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                codes[sym] = (next_code[len as usize], len);
                next_code[len as usize] += 1;
            }
        }

        // Step 4: build the two-level table
        // Calculate total table size (primary + all sub-tables)
        let primary_size = 1usize << primary_bits;
        // First pass: figure out how many sub-table entries we need
        let mut sub_offsets = vec![0u32; primary_size]; // temp: count of entries per sub-table root
        let mut total_sub = 0usize;

        if max_len > primary_bits {
            // Count how many codes fall into each sub-table bucket
            for &(code_val, len) in codes.iter() {
                if len > primary_bits {
                    let primary_idx = (reverse_bits(code_val, len) as usize) & (primary_size - 1);
                    sub_offsets[primary_idx] += 1;
                }
            }
            // Compute offsets
            let mut offset = primary_size;
            let sub_bits = max_len - primary_bits;
            let sub_size = 1usize << sub_bits;
            for slot in sub_offsets.iter_mut().take(primary_size) {
                if *slot > 0 {
                    let start = offset;
                    offset += sub_size;
                    *slot = start as u32;
                } else {
                    *slot = 0;
                }
            }
            total_sub = offset - primary_size;
        }

        let mut table = vec![0u32; primary_size + total_sub];

        // Fill entries
        for (sym, &(code_val, len)) in codes.iter().enumerate() {
            if len == 0 {
                continue;
            }

            let reversed = reverse_bits(code_val, len) as usize;

            if len <= primary_bits {
                // Direct entry: replicate across all matching bit patterns
                let e = entry(sym as u16, len);
                let step = 1 << len;
                let mut idx = reversed;
                while idx < primary_size {
                    table[idx] = e;
                    idx += step;
                }
            } else {
                // Sub-table entry
                let primary_idx = reversed & (primary_size - 1);
                let sub_base = sub_offsets[primary_idx] as usize;
                let sub_bits = max_len - primary_bits;

                // Write the redirect entry in primary table
                table[primary_idx] = (sub_base as u32) | ((sub_bits as u32) << 16) | SUB_TABLE_FLAG;

                // Fill sub-table entry
                let sub_idx = reversed >> primary_bits;
                let e = entry(sym as u16, len);
                let step = 1usize << (len - primary_bits);
                let sub_size = 1usize << sub_bits;
                let mut idx = sub_idx;
                while idx < sub_size {
                    table[sub_base + idx] = e;
                    idx += step;
                }
            }
        }

        Ok(Self {
            table,
            primary_bits,
        })
    }

    /// Decode one symbol from the bit reader.
    #[inline]
    fn decode(&self, reader: &mut BitReader) -> Result<u16, InflateError> {
        reader.fill(15); // max code length
        let idx = (reader.buf & ((1u32 << self.primary_bits) - 1)) as usize;
        let e = self.table[idx];

        if e & SUB_TABLE_FLAG == 0 {
            let len = entry_len(e);
            reader.consume(len);
            Ok(entry_sym(e))
        } else {
            let sub_base = (e & 0xFFFF) as usize;
            let sub_bits = entry_len(e); // stored in len field for redirects
            reader.consume(self.primary_bits);
            let sub_idx = (reader.buf & ((1u32 << sub_bits) - 1)) as usize;
            let e2 = self.table[sub_base + sub_idx];
            let len2 = entry_len(e2);
            // len2 includes primary_bits already as total length; we only consume the sub-table portion
            let sub_consumed = len2 - self.primary_bits;
            reader.consume(sub_consumed);
            Ok(entry_sym(e2))
        }
    }
}

/// Reverse the bottom `len` bits of `code`.
#[inline]
fn reverse_bits(code: u32, len: u8) -> u32 {
    let mut result = 0u32;
    let mut c = code;
    for _ in 0..len {
        result = (result << 1) | (c & 1);
        c >>= 1;
    }
    result
}

// ── Fixed Huffman tables ────────────────────────────────────────────────────
//
// The fixed lit/dist tables (RFC 1951 §3.2.6) are immutable and used by every
// BTYPE=1 block. Cache them in `OnceLock`s so repeated inflate calls (or
// streams with many fixed blocks) reuse the same allocations.

static FIXED_LIT_TABLE: OnceLock<HuffmanTable> = OnceLock::new();
static FIXED_DIST_TABLE: OnceLock<HuffmanTable> = OnceLock::new();

fn fixed_lit_table() -> &'static HuffmanTable {
    FIXED_LIT_TABLE.get_or_init(build_fixed_lit_table)
}

fn fixed_dist_table() -> &'static HuffmanTable {
    FIXED_DIST_TABLE.get_or_init(build_fixed_dist_table)
}

fn build_fixed_lit_table() -> HuffmanTable {
    let mut lengths = [0u8; 288];
    for l in lengths.iter_mut().take(144) {
        *l = 8;
    }
    for l in lengths.iter_mut().take(256).skip(144) {
        *l = 9;
    }
    for l in lengths.iter_mut().take(280).skip(256) {
        *l = 7;
    }
    for l in lengths.iter_mut().take(288).skip(280) {
        *l = 8;
    }
    HuffmanTable::build(&lengths, 9).unwrap()
}

fn build_fixed_dist_table() -> HuffmanTable {
    let lengths = [5u8; 32];
    HuffmanTable::build(&lengths, 5).unwrap()
}

// ── Core inflate logic ──────────────────────────────────────────────────────

/// Inflate a raw DEFLATE stream (no zlib/gzip wrapper) into `output`.
///
/// `max_size` caps `output.len()` after every write — the decoder aborts with
/// [`InflateError::OutputTooLarge`] before any push that would exceed it.
/// Callers should pass the declared uncompressed size from the ZIP header so
/// malformed streams cannot drive `Vec` growth past that bound.
pub(crate) fn inflate(
    input: &[u8],
    output: &mut Vec<u8>,
    max_size: usize,
) -> Result<(), InflateError> {
    let mut reader = BitReader::new(input);

    loop {
        let bfinal = reader.read(1);
        let btype = reader.read(2);

        // The block header may straddle the very last byte of input. We only
        // treat zero-padding as a hard error once we observe a non-final
        // block trying to start: legitimate streams must reach a `bfinal=1`
        // block whose end-of-block symbol arrived from real data.
        if reader.eof_consumed {
            return Err(InflateError::UnexpectedEof);
        }

        match btype {
            0 => inflate_stored(&mut reader, output, max_size)?,
            1 => inflate_huffman(
                &mut reader,
                output,
                fixed_lit_table(),
                fixed_dist_table(),
                max_size,
            )?,
            2 => inflate_dynamic(&mut reader, output, max_size)?,
            _ => return Err(InflateError::InvalidBlockType),
        }

        if bfinal != 0 {
            break;
        }
    }

    Ok(())
}

fn inflate_stored(
    reader: &mut BitReader,
    output: &mut Vec<u8>,
    max_size: usize,
) -> Result<(), InflateError> {
    reader.align();
    let len = reader.read(16) as usize;
    let nlen = reader.read(16) as usize;

    if len != (!nlen & 0xFFFF) {
        return Err(InflateError::InvalidStoredLen);
    }

    if output.len().saturating_add(len) > max_size {
        return Err(InflateError::OutputTooLarge);
    }

    let bytes = reader.read_bytes(len)?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn inflate_dynamic(
    reader: &mut BitReader,
    output: &mut Vec<u8>,
    max_size: usize,
) -> Result<(), InflateError> {
    let hlit = reader.read(5) as usize + 257;
    let hdist = reader.read(5) as usize + 1;
    let hclen = reader.read(4) as usize + 4;

    // Read code-length code lengths
    let mut codelen_lengths = [0u8; 19];
    for i in 0..hclen {
        codelen_lengths[CODELEN_ORDER[i]] = reader.read(3) as u8;
    }

    let codelen_table = HuffmanTable::build(&codelen_lengths, 7)?;

    // Decode literal/length + distance code lengths
    let total = hlit + hdist;
    let mut all_lengths = vec![0u8; total];
    let mut i = 0;

    while i < total {
        let sym = codelen_table.decode(reader)?;
        match sym {
            0..=15 => {
                all_lengths[i] = sym as u8;
                i += 1;
            }
            16 => {
                // Repeat previous length 3-6 times
                if i == 0 {
                    return Err(InflateError::InvalidHuffmanTable);
                }
                let repeat = reader.read(2) as usize + 3;
                let prev = all_lengths[i - 1];
                for _ in 0..repeat {
                    if i >= total {
                        return Err(InflateError::InvalidHuffmanTable);
                    }
                    all_lengths[i] = prev;
                    i += 1;
                }
            }
            17 => {
                // Repeat 0 for 3-10 times
                let repeat = reader.read(3) as usize + 3;
                i += repeat;
                if i > total {
                    return Err(InflateError::InvalidHuffmanTable);
                }
            }
            18 => {
                // Repeat 0 for 11-138 times
                let repeat = reader.read(7) as usize + 11;
                i += repeat;
                if i > total {
                    return Err(InflateError::InvalidHuffmanTable);
                }
            }
            _ => return Err(InflateError::InvalidHuffmanTable),
        }
    }

    let lit_table = HuffmanTable::build(&all_lengths[..hlit], 9)?;
    let dist_table = HuffmanTable::build(&all_lengths[hlit..], 6)?;

    inflate_huffman(reader, output, &lit_table, &dist_table, max_size)
}

fn inflate_huffman(
    reader: &mut BitReader,
    output: &mut Vec<u8>,
    lit_table: &HuffmanTable,
    dist_table: &HuffmanTable,
    max_size: usize,
) -> Result<(), InflateError> {
    loop {
        let sym = lit_table.decode(reader)?;

        // The end-of-block symbol (256) is the only legal way out of this
        // loop. If we just decoded a literal or back-ref *after* having
        // pulled zero-padding past the end of input, the decoded value is
        // undefined — bail before we emit garbage.
        if sym != 256 && reader.eof_consumed {
            return Err(InflateError::UnexpectedEof);
        }

        if sym < 256 {
            if output.len() >= max_size {
                return Err(InflateError::OutputTooLarge);
            }
            output.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            // Length-distance pair
            let len_idx = (sym - 257) as usize;
            if len_idx >= LENGTH_TABLE.len() {
                return Err(InflateError::InvalidLengthCode);
            }
            let (base_len, extra_bits) = LENGTH_TABLE[len_idx];
            let length = base_len as usize
                + if extra_bits > 0 {
                    reader.read(extra_bits) as usize
                } else {
                    0
                };

            let dist_sym = dist_table.decode(reader)?;
            let dist_idx = dist_sym as usize;
            if dist_idx >= DISTANCE_TABLE.len() {
                return Err(InflateError::InvalidDistanceCode);
            }
            let (base_dist, dist_extra) = DISTANCE_TABLE[dist_idx];
            let distance = base_dist as usize
                + if dist_extra > 0 {
                    reader.read(dist_extra) as usize
                } else {
                    0
                };

            if distance > output.len() {
                return Err(InflateError::DistanceTooFarBack);
            }

            if output.len().saturating_add(length) > max_size {
                return Err(InflateError::OutputTooLarge);
            }

            let start = output.len() - distance;
            if distance >= length {
                // Non-overlapping: source region is fully present, so we can
                // delegate to a vectorized memcpy.
                output.extend_from_within(start..start + length);
            } else {
                // Overlapping back-reference (e.g. RLE-style "AAAA…"): the
                // source grows as we write, so we have to step byte-by-byte.
                output.reserve(length);
                for i in 0..length {
                    let byte = output[start + i];
                    output.push(byte);
                }
            }
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn inflate_ok(compressed: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        inflate(compressed, &mut out, usize::MAX).expect("inflate failed");
        out
    }

    // ── Stored block (type 0) ──

    #[test]
    fn test_inflate_stored_block() {
        // Python: zlib.compress(b"stored data test", level=0, wbits=-15)
        let compressed: &[u8] = &[
            1, 16, 0, 239, 255, 115, 116, 111, 114, 101, 100, 32, 100, 97, 116, 97, 32, 116, 101,
            115, 116,
        ];
        assert_eq!(inflate_ok(compressed), b"stored data test");
    }

    // ── Fixed Huffman (type 1) ──

    #[test]
    fn test_inflate_fixed_huffman() {
        // Python: zlib.compress(b"Hello, dotLottie!", level=6, wbits=-15)
        let compressed: &[u8] = &[
            243, 72, 205, 201, 201, 215, 81, 72, 201, 47, 241, 201, 47, 41, 201, 76, 85, 4, 0,
        ];
        assert_eq!(inflate_ok(compressed), b"Hello, dotLottie!");
    }

    #[test]
    fn test_inflate_back_reference() {
        // "abcabcabcabcabcabcabcabc" — heavy LZ77 back-refs
        let compressed: &[u8] = &[75, 76, 74, 78, 196, 134, 0];
        assert_eq!(inflate_ok(compressed), b"abcabcabcabcabcabcabcabc");
    }

    #[test]
    fn test_inflate_repeat_single_byte() {
        // "A" * 500 — extreme overlapping back-reference (distance=1)
        let compressed: &[u8] = &[115, 116, 28, 5, 35, 13, 0, 0];
        let expected = vec![b'A'; 500];
        assert_eq!(inflate_ok(compressed), expected);
    }

    // ── Dynamic Huffman (type 2) ──

    #[test]
    fn test_inflate_dynamic_huffman() {
        // A lottie-like JSON snippet
        let compressed: &[u8] = &[
            45, 140, 59, 10, 128, 48, 16, 68, 239, 50, 245, 18, 140, 95, 216, 107, 88, 138, 69, 10,
            37, 65, 141, 98, 130, 34, 33, 119, 119, 65, 171, 55, 3, 51, 47, 193, 111, 96, 24, 239,
            54, 19, 221, 238, 65, 184, 164, 55, 170, 83, 90, 178, 59, 192, 5, 97, 23, 180, 194,
            249, 4, 87, 194, 27, 220, 232, 146, 96, 127, 174, 230, 153, 206, 0, 30, 18, 226, 3,
            174, 233, 179, 246, 214, 28, 147, 88, 130, 220, 52, 97, 145, 69, 202, 121, 204, 47,
        ];
        let expected = b"{\"nm\":\"animation\",\"v\":\"5.7.1\",\"ip\":0,\"op\":60,\"fr\":30,\"w\":512,\"h\":512,\"layers\":[{\"ty\":4,\"nm\":\"Shape\",\"sr\":1,\"ks\":{}}]}";
        assert_eq!(inflate_ok(compressed), expected);
    }

    // ── Empty ──

    #[test]
    fn test_inflate_empty() {
        // Python: zlib.compress(b"", level=6, wbits=-15)
        let compressed: &[u8] = &[3, 0];
        assert_eq!(inflate_ok(compressed), b"");
    }

    // ── Multiple blocks ──

    #[test]
    fn test_inflate_multiple_blocks() {
        // Two blocks via Z_FULL_FLUSH: "First block of data. " + "Second block of data."
        let compressed: &[u8] = &[
            114, 203, 44, 42, 46, 81, 72, 202, 201, 79, 206, 86, 200, 79, 83, 72, 73, 44, 73, 212,
            83, 0, 0, 0, 0, 255, 255, 11, 78, 77, 206, 207, 75, 81, 72, 202, 201, 79, 206, 86, 200,
            79, 83, 72, 73, 44, 73, 212, 3, 0,
        ];
        assert_eq!(
            inflate_ok(compressed),
            b"First block of data. Second block of data."
        );
    }

    // ── Error cases ──

    #[test]
    fn test_inflate_invalid_block_type() {
        // BFINAL=1, BTYPE=3 (reserved) → bits: 1 11 = 0b111 = 0x07
        let compressed: &[u8] = &[0x07];
        let mut out = Vec::new();
        assert!(matches!(
            inflate(compressed, &mut out, usize::MAX),
            Err(InflateError::InvalidBlockType)
        ));
    }

    #[test]
    fn test_inflate_bad_stored_len() {
        // Stored block with LEN/NLEN mismatch
        // BFINAL=1 BTYPE=00 → first byte = 0x01 (bits: 1 00 = stored, final)
        // Then LEN=5 (0x0005), NLEN=0x0000 (should be 0xFFFA)
        let compressed: &[u8] = &[0x01, 0x05, 0x00, 0x00, 0x00];
        let mut out = Vec::new();
        assert!(matches!(
            inflate(compressed, &mut out, usize::MAX),
            Err(InflateError::InvalidStoredLen)
        ));
    }

    // ── Size / EOF guards ──

    #[test]
    fn test_inflate_output_too_large_literals() {
        // Valid fixed-Huffman stream that decodes to "Hello, dotLottie!" (17 bytes).
        let compressed: &[u8] = &[
            243, 72, 205, 201, 201, 215, 81, 72, 201, 47, 241, 201, 47, 41, 201, 76, 85, 4, 0,
        ];
        let mut out = Vec::new();
        // Cap below the legitimate output length — decoder must abort.
        assert!(matches!(
            inflate(compressed, &mut out, 5),
            Err(InflateError::OutputTooLarge)
        ));
    }

    #[test]
    fn test_inflate_output_too_large_back_reference() {
        // "abcabcabcabcabcabcabcabc" — 24 bytes via heavy LZ77 back-refs.
        let compressed: &[u8] = &[75, 76, 74, 78, 196, 134, 0];
        let mut out = Vec::new();
        // Cap below the back-reference length — must abort before extend.
        assert!(matches!(
            inflate(compressed, &mut out, 10),
            Err(InflateError::OutputTooLarge)
        ));
    }

    #[test]
    fn test_inflate_truncated_huffman_stream() {
        // Take a valid stream and lop off its tail. Decoding past EOF must
        // surface UnexpectedEof rather than silently producing garbage.
        let compressed: &[u8] = &[243, 72, 205, 201, 201]; // truncated "Hello, dotLottie!"
        let mut out = Vec::new();
        let err = inflate(compressed, &mut out, usize::MAX).unwrap_err();
        assert!(
            matches!(
                err,
                InflateError::UnexpectedEof
                    | InflateError::OutputTooLarge
                    | InflateError::InvalidLengthCode
                    | InflateError::InvalidDistanceCode
            ),
            "expected truncation-related error, got {err:?}"
        );
    }

    // ── BitReader unit tests ──

    #[test]
    fn test_bit_reader_basic() {
        let data = [0b10110100u8, 0b01101001u8];
        let mut r = BitReader::new(&data);
        // LSB-first: first byte 0b10110100 → bits come out as 0,0,1,0,1,1,0,1
        assert_eq!(r.read(1), 0);
        assert_eq!(r.read(1), 0);
        assert_eq!(r.read(1), 1);
        assert_eq!(r.read(3), 0b110); // next 3 bits: 0,1,1 → LSB-first = 0b110
        assert_eq!(r.read(2), 0b10); // next 2 bits: 1,0 → 0b10
    }

    #[test]
    fn test_bit_reader_cross_byte() {
        let data = [0xFF, 0x00];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read(4), 0xF);
        // Next 8 bits span bytes: 4 bits of 0xFF upper + 4 bits of 0x00 lower
        assert_eq!(r.read(8), 0x0F); // 1111 0000 → LSB-first
    }

    #[test]
    fn test_bit_reader_align() {
        let data = [0xFF, 0xAB];
        let mut r = BitReader::new(&data);
        r.read(3); // consume 3 bits
        r.align(); // skip remaining 5 bits in first byte
        assert_eq!(r.read(8), 0xAB);
    }

    // ── Huffman table unit tests ──

    #[test]
    fn test_huffman_fixed_lit_decode() {
        let table = build_fixed_lit_table();
        // Encode literal 'A' (65) with fixed Huffman: code length 8, value 0x30+65=0xC1 → reversed
        // For fixed Huffman, literals 0-143 use 8-bit codes starting at 0b00110000
        // Symbol 65: code = 0b00110000 + 65 = 0b01110001 = 0x71, reversed = 0x8E
        let code: u32 = 0x30 + 65; // 0b01110001
        let reversed = reverse_bits(code, 8);
        let mut data = [0u8; 4];
        data[0] = reversed as u8;
        data[1] = (reversed >> 8) as u8;
        let mut reader = BitReader::new(&data);
        let sym = table.decode(&mut reader).unwrap();
        assert_eq!(sym, 65); // 'A'
    }

    #[test]
    fn test_huffman_build_simple() {
        // Canonical Huffman for lengths [1, 2, 2]:
        //   sym0: code=0 (1-bit), reversed=0
        //   sym1: code=10 (2-bit), reversed=01
        //   sym2: code=11 (2-bit), reversed=11
        // Bit stream (LSB-first in byte): 0, 01, 11 → byte = 0b_11_01_0 = 0b00011010
        let lengths = [1u8, 2, 2];
        let table = HuffmanTable::build(&lengths, 2).unwrap();
        let data = [0b00011010u8, 0x00];
        let mut reader = BitReader::new(&data);
        assert_eq!(table.decode(&mut reader).unwrap(), 0);
        assert_eq!(table.decode(&mut reader).unwrap(), 1);
        assert_eq!(table.decode(&mut reader).unwrap(), 2);
    }
}
