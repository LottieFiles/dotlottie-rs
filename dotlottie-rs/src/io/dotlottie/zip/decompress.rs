use super::inflate;
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
    _expected_crc32: u32,
    reuse_buf: &mut Vec<u8>,
) -> Result<Cow<'a, [u8]>, ZipError> {
    if uncompressed_size > MAX_UNCOMPRESSED_SIZE {
        return Err(ZipError::DecompressError);
    }

    match compression_method {
        METHOD_STORE => {
            if compressed.len() != uncompressed_size {
                return Err(ZipError::DecompressError);
            }
            Ok(Cow::Borrowed(compressed))
        }
        METHOD_DEFLATE => {
            reuse_buf.clear();
            reuse_buf.reserve(uncompressed_size);
            inflate::inflate(compressed, reuse_buf)
                .map_err(|_| ZipError::DecompressError)?;
            if reuse_buf.len() != uncompressed_size {
                return Err(ZipError::DecompressError);
            }
            Ok(Cow::Owned(std::mem::take(reuse_buf)))
        }
        other => Err(ZipError::UnsupportedCompression(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_stored() {
        let data = b"hello world";
        let mut buf = Vec::new();
        let result = decompress(data, METHOD_STORE, data.len(), 0, &mut buf).unwrap();
        assert_eq!(&*result, b"hello world");
        // Should be borrowed, not owned
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_decompress_stored_size_mismatch() {
        let data = b"hello";
        let mut buf = Vec::new();
        assert!(decompress(data, METHOD_STORE, 10, 0, &mut buf).is_err());
    }

    #[test]
    fn test_decompress_deflated() {
        // "Hello, dotLottie!" compressed with raw deflate
        let compressed: &[u8] = &[
            243, 72, 205, 201, 201, 215, 81, 72, 201, 47, 241, 201, 47, 41, 201, 76, 85, 4, 0,
        ];
        let mut buf = Vec::new();
        let result = decompress(compressed, METHOD_DEFLATE, 17, 0, &mut buf).unwrap();
        assert_eq!(&*result, b"Hello, dotLottie!");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn test_decompress_unsupported_method() {
        let mut buf = Vec::new();
        let err = decompress(b"", 99, 0, 0, &mut buf).unwrap_err();
        assert!(matches!(err, ZipError::UnsupportedCompression(99)));
    }

    #[test]
    fn test_decompress_too_large() {
        let mut buf = Vec::new();
        let err = decompress(b"", METHOD_DEFLATE, MAX_UNCOMPRESSED_SIZE + 1, 0, &mut buf).unwrap_err();
        assert!(matches!(err, ZipError::DecompressError));
    }
}
