#[derive(Debug)]
pub enum DotLottieError {
    ArchiveOpenError,
    StateMachineError,
    FileFindError,
    ReadContentError,
    MutexLockError,
    AnimationNotFound,
    AssetNotFound,
    AnimationsNotFound,
    ManifestNotFound,
    InvalidUtf8Error,
}
