use super::{LottieKeyframe, LottieProperty};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorValue(pub [f32; 3]);

impl<'de> Deserialize<'de> for ColorValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Array(arr) => {
                if arr.len() == 3 || arr.len() == 4 {
                    let r = arr[0]
                        .as_f64()
                        .ok_or_else(|| Error::custom("Color component must be a number"))?
                        as f32;
                    let g = arr[1]
                        .as_f64()
                        .ok_or_else(|| Error::custom("Color component must be a number"))?
                        as f32;
                    let b = arr[2]
                        .as_f64()
                        .ok_or_else(|| Error::custom("Color component must be a number"))?
                        as f32;
                    Ok(ColorValue([r, g, b]))
                } else {
                    Err(Error::custom(format!(
                        "Color array must have 3 or 4 elements, got {}",
                        arr.len()
                    )))
                }
            }
            _ => Err(Error::custom("Color must be an array")),
        }
    }
}

impl serde::Serialize for ColorValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.0[0])?;
        seq.serialize_element(&self.0[1])?;
        seq.serialize_element(&self.0[2])?;
        seq.end()
    }
}

pub type ColorSlot = LottieProperty<ColorValue>;

impl ColorSlot {
    pub fn new(color: [f32; 3]) -> Self {
        Self::static_value(ColorValue(color))
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<ColorValue>>) -> Self {
        Self::animated(keyframes)
    }
}
