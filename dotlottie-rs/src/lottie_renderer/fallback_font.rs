// The font is embedded as compressed data and decompressed at runtime using LZSS algorithm.

#[cfg(feature = "tvg-ttf")]
const COMPRESSED_FONT_SIZE: usize = 9721;
#[cfg(feature = "tvg-ttf")]
const DEFAULT_FONT_SIZE: usize = 14852;
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
