use super::crc32;
use super::error::ZipError;
use super::inflate;
use std::borrow::Cow;

const METHOD_STORE: u16 = 0;
const METHOD_DEFLATE: u16 = 8;

// 256 MB — generous upper bound for any single file in a dotLottie archive.
const MAX_UNCOMPRESSED_SIZE: usize = 256 * 1024 * 1024;

pub(crate) fn decompress(
    compressed: &[u8],
    compression_method: u16,
    uncompressed_size: usize,
    expected_crc32: u32,
) -> Result<Cow<'_, [u8]>, ZipError> {
    if uncompressed_size > MAX_UNCOMPRESSED_SIZE {
        return Err(ZipError::EntryTooLarge { uncompressed_size });
    }

    match compression_method {
        METHOD_STORE => {
            if compressed.len() != uncompressed_size {
                return Err(ZipError::UncompressedSizeMismatch {
                    expected: uncompressed_size,
                    actual: compressed.len(),
                });
            }
            verify_crc32(compressed, expected_crc32)?;
            Ok(Cow::Borrowed(compressed))
        }
        METHOD_DEFLATE => {
            // Reserve one extra byte so consumers that need a null sentinel
            // (e.g. ThorVG's Lottie parser) can push it without reallocating.
            let mut buf = Vec::with_capacity(uncompressed_size + 1);
            inflate::inflate(compressed, &mut buf).map_err(ZipError::Decompress)?;
            if buf.len() != uncompressed_size {
                return Err(ZipError::UncompressedSizeMismatch {
                    expected: uncompressed_size,
                    actual: buf.len(),
                });
            }
            verify_crc32(&buf, expected_crc32)?;
            Ok(Cow::Owned(buf))
        }
        other => Err(ZipError::UnsupportedCompression(other)),
    }
}

#[inline]
fn verify_crc32(data: &[u8], expected: u32) -> Result<(), ZipError> {
    let actual = crc32::crc32(data);
    if actual != expected {
        return Err(ZipError::ChecksumMismatch { expected, actual });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_stored() {
        let data = b"hello world";
        let crc = crc32::crc32(data);
        let result = decompress(data, METHOD_STORE, data.len(), crc).unwrap();
        assert_eq!(&*result, b"hello world");
        // Should be borrowed, not owned.
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_decompress_stored_size_mismatch() {
        let data = b"hello";
        let err = decompress(data, METHOD_STORE, 10, 0).unwrap_err();
        assert!(matches!(
            err,
            ZipError::UncompressedSizeMismatch {
                expected: 10,
                actual: 5
            }
        ));
    }

    #[test]
    fn test_decompress_deflated() {
        // "Hello, dotLottie!" compressed with raw deflate.
        let compressed: &[u8] = &[
            243, 72, 205, 201, 201, 215, 81, 72, 201, 47, 241, 201, 47, 41, 201, 76, 85, 4, 0,
        ];
        let plain = b"Hello, dotLottie!";
        let crc = crc32::crc32(plain);
        let result = decompress(compressed, METHOD_DEFLATE, plain.len(), crc).unwrap();
        assert_eq!(&*result, plain);
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn test_decompress_unsupported_method() {
        let err = decompress(b"", 99, 0, 0).unwrap_err();
        assert!(matches!(err, ZipError::UnsupportedCompression(99)));
    }

    #[test]
    fn test_decompress_too_large() {
        let err = decompress(b"", METHOD_DEFLATE, MAX_UNCOMPRESSED_SIZE + 1, 0).unwrap_err();
        assert!(matches!(err, ZipError::EntryTooLarge { .. }));
    }

    #[test]
    fn test_decompress_stored_crc_mismatch() {
        let data = b"hello world";
        let err = decompress(data, METHOD_STORE, data.len(), 0).unwrap_err();
        assert!(matches!(err, ZipError::ChecksumMismatch { .. }));
    }
}
