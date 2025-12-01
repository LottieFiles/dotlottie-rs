use super::{LottieKeyframe, LottieProperty};

pub type ColorSlot = LottieProperty<[f32; 4]>;

impl ColorSlot {
    pub fn new(color: [f32; 4]) -> Self {
        Self::static_value(color)
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<[f32; 4]>>) -> Self {
        Self::animated(keyframes)
    }
}
