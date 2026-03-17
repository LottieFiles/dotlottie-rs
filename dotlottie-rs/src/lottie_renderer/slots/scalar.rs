use super::{LottieKeyframe, LottieProperty};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarValue(pub f32);

impl<'de> Deserialize<'de> for ScalarValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Number(n) => n
                .as_f64()
                .map(|f| ScalarValue(f as f32))
                .ok_or_else(|| Error::custom("Invalid number for scalar")),
            Value::Array(arr) => {
                if arr.len() == 1 {
                    if let Some(Value::Number(n)) = arr.first() {
                        n.as_f64()
                            .map(|f| ScalarValue(f as f32))
                            .ok_or_else(|| Error::custom("Invalid number in scalar array"))
                    } else {
                        Err(Error::custom("Scalar array must contain a number"))
                    }
                } else {
                    Err(Error::custom(format!(
                        "Scalar array must have exactly 1 element, got {}",
                        arr.len()
                    )))
                }
            }
            _ => Err(Error::custom(
                "Scalar must be a number or single-element array",
            )),
        }
    }
}

impl serde::Serialize for ScalarValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

pub type ScalarSlot = LottieProperty<ScalarValue>;

impl ScalarSlot {
    pub fn new(value: f32) -> Self {
        Self::static_value(ScalarValue(value))
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<ScalarValue>>) -> Self {
        Self::animated(keyframes)
    }
}
