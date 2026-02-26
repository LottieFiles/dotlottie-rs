use super::parse::ZipError;
use std::borrow::Cow;

const METHOD_STORE: u16 = 0;
const METHOD_DEFLATE: u16 = 8;

// 256 MB — generous upper bound for any single file in a dotLottie archive
const MAX_UNCOMPRESSED_SIZE: usize = 256 * 1024 * 1024;

pub(crate) fn decompress<'a>(
    compressed: &'a [u8],
    compression_method: u16,
    uncompressed_size: usize,
    expected_crc32: u32,
    decompressor: &mut libdeflater::Decompressor,
    reuse_buf: &mut Vec<u8>,
) -> Result<Cow<'a, [u8]>, ZipError> {
    if uncompressed_size > MAX_UNCOMPRESSED_SIZE {
        return Err(ZipError::DecompressError);
    }

    let output: Cow<'a, [u8]> = match compression_method {
        METHOD_STORE => {
            if compressed.len() != uncompressed_size {
                return Err(ZipError::DecompressError);
            }
            Cow::Borrowed(compressed)
        }
        METHOD_DEFLATE => {
            reuse_buf.resize(uncompressed_size, 0);
            let actual_size = decompressor
                .deflate_decompress(compressed, reuse_buf)
                .map_err(|_| ZipError::DecompressError)?;
            if actual_size != uncompressed_size {
                return Err(ZipError::DecompressError);
            }
            // Clone valid data out; reuse_buf retains its capacity for next call
            Cow::Owned(reuse_buf[..actual_size].to_vec())
        }
        other => return Err(ZipError::UnsupportedCompression(other)),
    };

    let mut hasher = libdeflater::Crc::new();
    hasher.update(&output);
    if hasher.sum() != expected_crc32 {
        return Err(ZipError::Crc32Mismatch);
    }

    Ok(output)
}
