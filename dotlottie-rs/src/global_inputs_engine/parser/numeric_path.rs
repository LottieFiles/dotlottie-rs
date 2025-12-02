use crate::{slots::PropertyValue, LottieRenderer, ScalarSlot, TextSlot};

#[derive(Debug, Clone, Default)]
pub enum NumericPath {
    // Numeric -> Scalar target
    #[default]
    StaticValue,
    Keyframe(usize),

    // Numeric -> TextDocument targets
    FontSize,                     // value/fontSize
    StrokeWidth,                  // value/strokeWidth
    LineHeight,                   // value/lineHeight
    Tracking,                     // value/tracking
    BaselineShift,                // value/baselineShift
    KeyframeFontSize(usize),      // keyframes/{kf}/value/fontSize
    KeyframeStrokeWidth(usize),   // keyframes/{kf}/value/strokeWidth
    KeyframeLineHeight(usize),    // keyframes/{kf}/value/lineHeight
    KeyframeTracking(usize),      // keyframes/{kf}/value/tracking
    KeyframeBaselineShift(usize), // keyframes/{kf}/value/baselineShift
}

impl NumericPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            // Numeric -> Scalar
            ["value"] => Ok(NumericPath::StaticValue),
            ["keyframes", idx, "value"] => {
                let i: usize = idx.parse().map_err(|_| format!("invalid index: {idx}"))?;
                Ok(NumericPath::Keyframe(i))
            }

            // Numeric -> TextDocument (static)
            ["value", "fontSize"] => Ok(NumericPath::FontSize),
            ["value", "strokeWidth"] => Ok(NumericPath::StrokeWidth),
            ["value", "lineHeight"] => Ok(NumericPath::LineHeight),
            ["value", "tracking"] => Ok(NumericPath::Tracking),
            ["value", "baselineShift"] => Ok(NumericPath::BaselineShift),

            // Numeric -> TextDocument (animated)
            ["keyframes", kf_idx, "value", "fontSize"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(NumericPath::KeyframeFontSize(kf))
            }
            ["keyframes", kf_idx, "value", "strokeWidth"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(NumericPath::KeyframeStrokeWidth(kf))
            }
            ["keyframes", kf_idx, "value", "lineHeight"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(NumericPath::KeyframeLineHeight(kf))
            }
            ["keyframes", kf_idx, "value", "tracking"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(NumericPath::KeyframeTracking(kf))
            }
            ["keyframes", kf_idx, "value", "baselineShift"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(NumericPath::KeyframeBaselineShift(kf))
            }

            _ => Err(format!("invalid numeric path: {path}")),
        }
    }

    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: f32,
    ) -> Result<(), String> {
        if self.targets_text() {
            let text_slot = renderer
                .get_text_slot(rule_id)
                .ok_or_else(|| format!("text slot '{}' not found", rule_id))?;
            self.apply_to_text(text_slot, value)
        } else {
            let scalar_slot = renderer
                .get_scalar_slot(rule_id)
                .ok_or_else(|| format!("scalar slot '{}' not found", rule_id))?;
            self.apply_to_scalar(scalar_slot, value)
        }
    }

    pub fn apply_to_scalar(&self, slot: &mut ScalarSlot, value: f32) -> Result<(), String> {
        match self {
            NumericPath::StaticValue => {
                slot.value = PropertyValue::Static(value);
                Ok(())
            }
            NumericPath::Keyframe(keyframe) => match &mut slot.value {
                PropertyValue::Animated(keyframes) => {
                    let kf = keyframes
                        .get_mut(*keyframe)
                        .ok_or_else(|| format!("index {keyframe} out of bounds"))?;
                    kf.start_value = value;
                    Ok(())
                }
                PropertyValue::Static(_) => Err(
                    "Numeric binding path was for animated slot, but slot is static".to_string(),
                ),
            },
            _ => Err("path targets text slot, not scalar slot".to_string()),
        }
    }

    pub fn apply_to_text(&self, slot: &mut TextSlot, value: f32) -> Result<(), String> {
        match self {
            NumericPath::FontSize => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.font_size = Some(value);
                Ok(())
            }
            NumericPath::StrokeWidth => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.stroke_width = Some(value);
                Ok(())
            }
            NumericPath::LineHeight => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.line_height = Some(value);
                Ok(())
            }
            NumericPath::Tracking => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.tracking = Some(value);
                Ok(())
            }
            NumericPath::BaselineShift => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.baseline_shift = Some(value);
                Ok(())
            }
            NumericPath::KeyframeFontSize(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.font_size = Some(value);
                Ok(())
            }
            NumericPath::KeyframeStrokeWidth(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.stroke_width = Some(value);
                Ok(())
            }
            NumericPath::KeyframeLineHeight(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.line_height = Some(value);
                Ok(())
            }
            NumericPath::KeyframeTracking(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.tracking = Some(value);
                Ok(())
            }
            NumericPath::KeyframeBaselineShift(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.baseline_shift = Some(value);
                Ok(())
            }
            NumericPath::StaticValue | NumericPath::Keyframe(_) => {
                Err("path targets scalar slot, not text slot".to_string())
            }
        }
    }

    /// Returns true if this path targets a text slot
    pub fn targets_text(&self) -> bool {
        matches!(
            self,
            NumericPath::FontSize
                | NumericPath::StrokeWidth
                | NumericPath::LineHeight
                | NumericPath::Tracking
                | NumericPath::BaselineShift
                | NumericPath::KeyframeFontSize(_)
                | NumericPath::KeyframeStrokeWidth(_)
                | NumericPath::KeyframeLineHeight(_)
                | NumericPath::KeyframeTracking(_)
                | NumericPath::KeyframeBaselineShift(_)
        )
    }
}
