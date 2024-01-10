use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum DotLottieError {
    #[error("Failed to open archive")]
    ArchiveOpenError,

    #[error("Unable to find the file: {file_name}")]
    FileFindError { file_name: String },

    #[error("Unable to read the contents")]
    ReadContentError,

    #[error("Unable to load Lottie animation")]
    LoadContentError,
}