use super::{LottieKeyframe, LottieProperty};

pub type ColorSlot = LottieProperty<[f32; 3]>;

impl ColorSlot {
    pub fn new(color: [f32; 3]) -> Self {
        Self::static_value(color)
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<[f32; 3]>>) -> Self {
        Self::animated(keyframes)
    }
}
