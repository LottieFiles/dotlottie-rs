use crate::LottieRendererError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum DotLottieResult {
    Success = 0,
    Error = 1,
    InvalidParameter = 2,
    ManifestNotAvailable = 3,
    AnimationNotLoaded = 4,
    InsufficientCondition = 5,
}

impl DotLottieResult {
    pub fn is_ok(self) -> bool {
        self == DotLottieResult::Success
    }

    pub fn is_err(self) -> bool {
        self != DotLottieResult::Success
    }
}

impl From<LottieRendererError> for DotLottieResult {
    fn from(err: LottieRendererError) -> Self {
        match err {
            LottieRendererError::InvalidArgument | LottieRendererError::InvalidColor => {
                DotLottieResult::InvalidParameter
            }
            LottieRendererError::AnimationNotLoaded => DotLottieResult::AnimationNotLoaded,
            _ => DotLottieResult::Error,
        }
    }
}

impl<E: Into<DotLottieResult>> From<Result<(), E>> for DotLottieResult {
    fn from(result: Result<(), E>) -> Self {
        match result {
            Ok(()) => DotLottieResult::Success,
            Err(e) => e.into(),
        }
    }
}
