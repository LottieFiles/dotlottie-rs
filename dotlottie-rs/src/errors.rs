use std::fmt;

use libdeflater::DecompressionError;

#[derive(Debug)]
pub enum DotLottiePlayerError {
    TvgInvalidArgument { function_name: String },
    TvgInsufficientCondition { function_name: String },
    TvgFailedAllocation { function_name: String },
    TvgMemoryCorruption { function_name: String },
    TvgNotSupported { function_name: String },
    TvgUnknown { function_name: String },
    InvalidColor(String),
    InvalidArgument(String),
    ArchiveOpenError,
    FileFindError { file_name: String },
    ReadContentError,
    MutexLockError,
    AnimationNotFound { animation_id: String },
    AnimationsNotFound,
    ManifestNotFound,
    InvalidUtf8Error,
    InvalidDotLottieFile,
    DecompressionError(DecompressionError),
    Utf8Error(std::str::Utf8Error),
    IOError(std::io::Error),
    FromBytesError(std::array::TryFromSliceError),
    DataUnavailable,
}

impl fmt::Display for DotLottiePlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DotLottiePlayerError::TvgInvalidArgument { function_name } => {
                write!(f, "Invalid argument provided in {}", function_name)
            }
            DotLottiePlayerError::TvgInsufficientCondition { function_name } => {
                write!(f, "Insufficient condition in {}", function_name)
            }
            DotLottiePlayerError::TvgFailedAllocation { function_name } => {
                write!(f, "Failed memory allocation in {}", function_name)
            }
            DotLottiePlayerError::TvgMemoryCorruption { function_name } => {
                write!(f, "Memory corruption detected in {}", function_name)
            }
            DotLottiePlayerError::TvgNotSupported { function_name } => {
                write!(f, "Operation not supported in {}", function_name)
            }
            DotLottiePlayerError::TvgUnknown { function_name } => {
                write!(f, "Unknown error occurred in {}", function_name)
            }
            DotLottiePlayerError::InvalidColor(color) => write!(f, "Invalid color: {}", color),
            DotLottiePlayerError::InvalidArgument(argument) => {
                write!(f, "Invalid argument: {}", argument)
            }
            DotLottiePlayerError::ArchiveOpenError => write!(f, "Failed to open archive"),
            DotLottiePlayerError::FileFindError { file_name } => {
                write!(f, "Unable to find the file: {}", file_name)
            }
            DotLottiePlayerError::ReadContentError => write!(f, "Unable to read the contents"),
            DotLottiePlayerError::MutexLockError => {
                write!(f, "Unable to lock the animations mutex")
            }
            DotLottiePlayerError::AnimationNotFound { animation_id } => {
                write!(f, "Animation not found: {}", animation_id)
            }
            DotLottiePlayerError::AnimationsNotFound => {
                write!(f, "No animations found in dotLottie file")
            }
            DotLottiePlayerError::ManifestNotFound => write!(f, "No manifest found"),
            DotLottiePlayerError::InvalidUtf8Error => write!(f, "Invalid UTF-8"),
            DotLottiePlayerError::InvalidDotLottieFile => write!(f, "Invalid dotLottie file"),
            DotLottiePlayerError::DecompressionError(err) => {
                write!(f, "Decompression error: {}", err)
            }
            DotLottiePlayerError::Utf8Error(err) => write!(f, "UTF-8 error: {}", err),
            DotLottiePlayerError::IOError(err) => write!(f, "IO error: {}", err),
            DotLottiePlayerError::FromBytesError(err) => write!(f, "From bytes error: {}", err),
            DotLottiePlayerError::DataUnavailable => write!(f, "Data unavailable"),
        }
    }
}

impl std::error::Error for DotLottiePlayerError {}

// Implement From traits for conversion of errors
impl From<std::io::Error> for DotLottiePlayerError {
    fn from(err: std::io::Error) -> Self {
        DotLottiePlayerError::IOError(err)
    }
}

impl From<DecompressionError> for DotLottiePlayerError {
    fn from(err: DecompressionError) -> Self {
        DotLottiePlayerError::DecompressionError(err)
    }
}

impl From<std::str::Utf8Error> for DotLottiePlayerError {
    fn from(err: std::str::Utf8Error) -> Self {
        DotLottiePlayerError::Utf8Error(err)
    }
}

impl From<std::array::TryFromSliceError> for DotLottiePlayerError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        DotLottiePlayerError::FromBytesError(err)
    }
}
