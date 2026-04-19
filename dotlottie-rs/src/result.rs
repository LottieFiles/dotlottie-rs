use crate::LottieRendererError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerError {
    Unknown,
    InvalidParameter,
    ManifestNotAvailable,
    AnimationNotLoaded,
    InsufficientCondition,
}

impl From<LottieRendererError> for PlayerError {
    fn from(err: LottieRendererError) -> Self {
        match err {
            LottieRendererError::InvalidArgument | LottieRendererError::InvalidColor => {
                PlayerError::InvalidParameter
            }
            LottieRendererError::AnimationNotLoaded => PlayerError::AnimationNotLoaded,
            _ => PlayerError::Unknown,
        }
    }
}
