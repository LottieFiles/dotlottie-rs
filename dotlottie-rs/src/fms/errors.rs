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
    /// Archive is encrypted but no usable password was supplied.
    EncryptedArchive,
    /// A password was supplied but it did not decrypt the archive.
    InvalidPassword,
}
