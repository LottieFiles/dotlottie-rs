#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("failed to open archive")]
    ArchiveOpenError,
    #[error("file not found in archive")]
    FileFindError,
    #[error("failed to read archive content")]
    ReadContentError,
    #[error("no animations in archive")]
    AnimationsNotFound,
    #[error("invalid utf-8 content")]
    InvalidUtf8Error,
}
