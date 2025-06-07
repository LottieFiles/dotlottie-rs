#[derive(Debug)]
pub enum DotLottieError {
    ArchiveOpenError,
    StateMachineError,
    FileFindError,
    ReadContentError,
    MutexLockError,
    AnimationNotFound,
    AnimationsNotFound,
    ManifestNotFound,
    InvalidUtf8Error,
}
