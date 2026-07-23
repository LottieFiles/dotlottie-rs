use super::{LottieKeyframe, LottieProperty};
use crate::json::{write_f32, Value};
use crate::renderer::slots::SlotValue;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarValue(pub f32);

pub type ScalarSlot = LottieProperty<ScalarValue>;

impl ScalarSlot {
    pub fn new(value: f32) -> Self {
        Self::static_value(ScalarValue(value))
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<ScalarValue>>) -> Self {
        Self::animated(keyframes)
    }
}

impl SlotValue for ScalarValue {
    fn from_json(v: &Value) -> Option<Self> {
        if let Some(n) = v.as_f32() {
            return Some(ScalarValue(n));
        }
        let arr = v.as_array()?;
        if arr.len() == 1 {
            arr[0].as_f32().map(ScalarValue)
        } else {
            None
        }
    }
    fn write(&self, out: &mut String) {
        write_f32(self.0, out);
    }
}
