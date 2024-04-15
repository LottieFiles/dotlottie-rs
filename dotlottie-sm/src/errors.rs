use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum StateMachineError {
    #[error("Failed to parse JSON state machine definition")]
    ParsingError { reason: String },
    // #[error("Unable to find the file: {file_name}")]
    // FileFindError { file_name: String },

    // #[error("Unable to read the contents")]
    // ReadContentError,

    // #[error("Unable to lock the animations mutex")]
    // MutexLockError,

    // #[error("Animation not found")]
    // AnimationNotFound { animation_id: String },

    // #[error("No animations found in dotLottie file")]
    // AnimationsNotFound,

    // #[error("No manifest found")]
    // ManifestNotFound,

    // #[error("Invalid UTF-8")]
    // InvalidUtf8Error,
}
