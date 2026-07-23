use super::{LottieKeyframe, LottieProperty};
use crate::json::Value;
use crate::renderer::slots::{write_f32_slice, SlotValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorValue(pub [f32; 3]);

pub type ColorSlot = LottieProperty<ColorValue>;

impl ColorSlot {
    pub fn new(color: [f32; 3]) -> Self {
        Self::static_value(ColorValue(color))
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<ColorValue>>) -> Self {
        Self::animated(keyframes)
    }
}

impl SlotValue for ColorValue {
    fn from_json(v: &Value) -> Option<Self> {
        let arr = v.as_array()?;
        if arr.len() == 3 || arr.len() == 4 {
            Some(ColorValue([
                arr[0].as_f32()?,
                arr[1].as_f32()?,
                arr[2].as_f32()?,
            ]))
        } else {
            None
        }
    }
    fn write(&self, out: &mut String) {
        write_f32_slice(&self.0, out);
    }
}
