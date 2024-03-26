use thiserror::Error;

// TODO: consider replace thiserror with custom error handling solution or anyhow for binary size reduction

#[derive(Error, Debug)]
pub enum DotLottiePlayerError {
    // thorvg errors
    #[error("Invalid argument provided in {function_name}")]
    TvgInvalidArgument { function_name: String },
    #[error("Insufficient condition in {function_name}")]
    TvgInsufficientCondition { function_name: String },
    #[error("Failed memory allocation in {function_name}")]
    TvgFailedAllocation { function_name: String },
    #[error("Memory corruption detected in {function_name}")]
    TvgMemoryCorruption { function_name: String },
    #[error("Operation not supported in {function_name}")]
    TvgNotSupported { function_name: String },
    #[error("Unknown error occurred in {function_name}")]
    TvgUnknown { function_name: String },

    // lottie_renderer errors
    #[error("Invalid color: {0}")]
    InvalidColor(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    // dotlottie_loader errors
    #[error("Failed to open archive")]
    ArchiveOpenError,
    #[error("Unable to find the file: {file_name}")]
    FileFindError { file_name: String },
    #[error("Unable to read the contents")]
    ReadContentError,
    #[error("Unable to lock the animations mutex")]
    MutexLockError,
    #[error("Animation not found")]
    AnimationNotFound { animation_id: String },
    #[error("No animations found in dotLottie file")]
    AnimationsNotFound,
    #[error("No manifest found")]
    ManifestNotFound,
    #[error("Invalid UTF-8")]
    InvalidUtf8Error,
}
