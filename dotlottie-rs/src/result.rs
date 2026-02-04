use crate::LottieRendererError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DotLottiePlayerError {
    Unknown,
    InvalidParameter,
    ManifestNotAvailable,
    AnimationNotLoaded,
    InsufficientCondition,
}

impl From<LottieRendererError> for DotLottiePlayerError {
    fn from(err: LottieRendererError) -> Self {
        match err {
            LottieRendererError::InvalidArgument | LottieRendererError::InvalidColor => {
                DotLottiePlayerError::InvalidParameter
            }
            LottieRendererError::AnimationNotLoaded => DotLottiePlayerError::AnimationNotLoaded,
            _ => DotLottiePlayerError::Unknown,
        }
    }
}
