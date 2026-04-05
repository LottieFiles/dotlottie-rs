//! Minimal raw DEFLATE (RFC 1951) inflater optimized for dotLottie archives.
//!
//! Supports all three block types: stored (0), fixed Huffman (1), dynamic Huffman (2).
//! Uses two-level lookup tables for fast Huffman decoding.

use std::fmt;

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) enum InflateError {
    InvalidBlockType,
    InvalidHuffmanTable,
    InvalidStoredLen,
    InvalidLengthCode,
    InvalidDistanceCode,
    DistanceTooFarBack,
    UnexpectedEof,
}

impl fmt::Display for InflateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

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
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            buf: 0,
            bits: 0,
        }
    }

    /// Fill the bit buffer so it contains at least `need` bits (max 25).
    #[inline]
    fn fill(&mut self, need: u8) {
        while self.bits < need {
            let byte = if self.pos < self.data.len() {
                self.data[self.pos]
            } else {
                0 // reads past end produce zeros; callers check EOF via results
            };
            self.pos += 1;
            self.buf |= (byte as u32) << self.bits;
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
            for (_, &(code_val, len)) in codes.iter().enumerate() {
                if len > primary_bits {
                    let primary_idx = (reverse_bits(code_val, len) as usize) & (primary_size - 1);
                    sub_offsets[primary_idx] += 1;
                }
            }
            // Compute offsets
            let mut offset = primary_size;
            for i in 0..primary_size {
                if sub_offsets[i] > 0 {
                    let sub_bits = max_len - primary_bits;
                    let sub_size = 1usize << sub_bits;
                    let start = offset;
                    offset += sub_size;
                    sub_offsets[i] = start as u32;
                } else {
                    sub_offsets[i] = 0;
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
pub(crate) fn inflate(input: &[u8], output: &mut Vec<u8>) -> Result<(), InflateError> {
    let mut reader = BitReader::new(input);

    loop {
        let bfinal = reader.read(1);
        let btype = reader.read(2);

        match btype {
            0 => inflate_stored(&mut reader, output)?,
            1 => {
                let lit_table = build_fixed_lit_table();
                let dist_table = build_fixed_dist_table();
                inflate_huffman(&mut reader, output, &lit_table, &dist_table)?;
            }
            2 => inflate_dynamic(&mut reader, output)?,
            _ => return Err(InflateError::InvalidBlockType),
        }

        if bfinal != 0 {
            break;
        }
    }

    Ok(())
}

fn inflate_stored(reader: &mut BitReader, output: &mut Vec<u8>) -> Result<(), InflateError> {
    reader.align();
    let len = reader.read(16) as usize;
    let nlen = reader.read(16) as usize;

    if len != (!nlen & 0xFFFF) {
        return Err(InflateError::InvalidStoredLen);
    }

    let bytes = reader.read_bytes(len)?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn inflate_dynamic(reader: &mut BitReader, output: &mut Vec<u8>) -> Result<(), InflateError> {
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

    inflate_huffman(reader, output, &lit_table, &dist_table)
}

fn inflate_huffman(
    reader: &mut BitReader,
    output: &mut Vec<u8>,
    lit_table: &HuffmanTable,
    dist_table: &HuffmanTable,
) -> Result<(), InflateError> {
    loop {
        let sym = lit_table.decode(reader)?;

        if sym < 256 {
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
            let length = base_len as usize + if extra_bits > 0 {
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
            let distance = base_dist as usize + if dist_extra > 0 {
                reader.read(dist_extra) as usize
            } else {
                0
            };

            if distance > output.len() {
                return Err(InflateError::DistanceTooFarBack);
            }

            // Copy byte-by-byte to handle overlapping (distance < length)
            let start = output.len() - distance;
            for i in 0..length {
                let byte = output[start + i];
                output.push(byte);
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
        inflate(compressed, &mut out).expect("inflate failed");
        out
    }

    // ── Stored block (type 0) ──

    #[test]
    fn test_inflate_stored_block() {
        // Python: zlib.compress(b"stored data test", level=0, wbits=-15)
        let compressed: &[u8] = &[
            1, 16, 0, 239, 255, 115, 116, 111, 114, 101, 100, 32, 100, 97, 116, 97, 32, 116,
            101, 115, 116,
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
            45, 140, 59, 10, 128, 48, 16, 68, 239, 50, 245, 18, 140, 95, 216, 107, 88, 138, 69,
            10, 37, 65, 141, 98, 130, 34, 33, 119, 119, 65, 171, 55, 3, 51, 47, 193, 111, 96,
            24, 239, 54, 19, 221, 238, 65, 184, 164, 55, 170, 83, 90, 178, 59, 192, 5, 97, 23,
            180, 194, 249, 4, 87, 194, 27, 220, 232, 146, 96, 127, 174, 230, 153, 206, 0, 30,
            18, 226, 3, 174, 233, 179, 246, 214, 28, 147, 88, 130, 220, 52, 97, 145, 69, 202,
            121, 204, 47,
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
            114, 203, 44, 42, 46, 81, 72, 202, 201, 79, 206, 86, 200, 79, 83, 72, 73, 44, 73,
            212, 83, 0, 0, 0, 0, 255, 255, 11, 78, 77, 206, 207, 75, 81, 72, 202, 201, 79, 206,
            86, 200, 79, 83, 72, 73, 44, 73, 212, 3, 0,
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
            inflate(compressed, &mut out),
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
            inflate(compressed, &mut out),
            Err(InflateError::InvalidStoredLen)
        ));
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
