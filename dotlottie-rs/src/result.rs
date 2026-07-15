use crate::LottieRendererError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PlayerError {
    #[error("unknown error")]
    Unknown,
    #[error("invalid parameter")]
    InvalidParameter,
    #[error("no animation loaded")]
    AnimationNotLoaded,
    #[error("insufficient condition")]
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
