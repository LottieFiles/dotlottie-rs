#[derive(Debug)]
pub enum DotLottieError {
    ArchiveOpenError,
    FileFindError,
    ReadContentError,
    AnimationsNotFound,
    InvalidUtf8Error,
}
