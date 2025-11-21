use super::{LottieKeyframe, LottieProperty};

pub type ScalarSlot = LottieProperty<f32>;

impl ScalarSlot {
    pub fn new(value: f32) -> Self {
        Self::static_value(value)
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<f32>>) -> Self {
        Self::animated(keyframes)
    }
}
