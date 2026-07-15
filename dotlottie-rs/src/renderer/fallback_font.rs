// Fallback font: DM Sans Regular (Google Fonts, OFL license)
// Characters: U+0020..U+007E (95 ASCII printable characters)
// Compression: LZSS (custom, see decompressor below)

#[cfg(feature = "tvg-ttf")]
const COMPRESSED_FONT_SIZE: usize = 6887;
#[cfg(feature = "tvg-ttf")]
const DEFAULT_FONT_SIZE: usize = 8732;
#[cfg(feature = "tvg-ttf")]
const COMPRESSED_FONT: &[u8] = include_bytes!("fallback_font.bin");
#[cfg(feature = "tvg-ttf")]
const DEFAULT_FONT_NAME: &str = "default";

#[cfg(feature = "tvg-ttf")]
pub fn font() -> (&'static str, Vec<u8>) {
    let mut output = vec![0u8; DEFAULT_FONT_SIZE];
    let mut input_pos = 0;
    let mut output_pos = 0;

    while input_pos < COMPRESSED_FONT_SIZE && output_pos < DEFAULT_FONT_SIZE {
        let flags = COMPRESSED_FONT[input_pos];
        input_pos += 1;

        for bit in 0..8 {
            if output_pos >= DEFAULT_FONT_SIZE {
                break;
            }

            if (flags & (1 << bit)) != 0 {
                // Literal byte
                output[output_pos] = COMPRESSED_FONT[input_pos];
                output_pos += 1;
                input_pos += 1;
            } else {
                // Back reference
                let offset_high = (COMPRESSED_FONT[input_pos] as u16) << 4;
                input_pos += 1;
                let length_offset = COMPRESSED_FONT[input_pos];
                input_pos += 1;

                let offset = offset_high | ((length_offset >> 4) as u16);
                let length = ((length_offset & 0x0F) + 3) as usize;

                let copy_pos = output_pos - offset as usize;
                for i in 0..length {
                    if output_pos >= DEFAULT_FONT_SIZE {
                        break;
                    }
                    output[output_pos] = output[copy_pos + i];
                    output_pos += 1;
                }
            }
        }
    }

    (DEFAULT_FONT_NAME, output)
}

#[cfg(test)]
#[cfg(feature = "tvg-ttf")]
mod tests {
    use super::*;

    #[test]
    fn test_font_decompresses_to_valid_ttf() {
        let (name, data) = font();
        assert_eq!(name, "default");
        assert!(
            data.len() >= DEFAULT_FONT_SIZE,
            "Font data too small: {} bytes",
            data.len()
        );
        // TrueType fonts start with version 0x00010000
        assert_eq!(
            &data[0..4],
            b"\x00\x01\x00\x00",
            "Invalid TTF header: {:?}",
            &data[0..4]
        );
    }
}
