use crate::{slots::PropertyValue, LottieRenderer, TextSlot, VectorSlot};

#[derive(Debug, Clone, Default)]
pub enum VectorPath {
    // Vector -> Vector target
    #[default]
    StaticValue,
    Keyframe(usize),

    // Vector -> TextDocument targets
    WrapSize,                    // value/wrapSize
    WrapPosition,                // value/wrapPosition
    KeyframeWrapSize(usize),     // keyframes/{kf}/value/wrapSize
    KeyframeWrapPosition(usize), // keyframes/{kf}/value/wrapPosition
}

impl VectorPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            // Vector/Position static value
            ["value"] => Ok(VectorPath::StaticValue),

            // Vector/Position keyframe
            ["keyframes", idx, "value"] => {
                let i: usize = idx.parse().map_err(|_| format!("invalid index: {idx}"))?;
                Ok(VectorPath::Keyframe(i))
            }

            // Vector -> TextDocument (static)
            ["value", "wrapSize"] => Ok(VectorPath::WrapSize),
            ["value", "wrapPosition"] => Ok(VectorPath::WrapPosition),

            // Vector -> TextDocument (animated)
            ["keyframes", kf_idx, "value", "wrapSize"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(VectorPath::KeyframeWrapSize(kf))
            }
            ["keyframes", kf_idx, "value", "wrapPosition"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(VectorPath::KeyframeWrapPosition(kf))
            }

            _ => Err(format!("invalid vector path: {path}")),
        }
    }

    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: &[f32],
    ) -> Result<(), String> {
        if self.targets_text() {
            let text_slot = renderer
                .get_text_slot(rule_id)
                .ok_or_else(|| format!("text slot '{}' not found", rule_id))?;
            self.apply_to_text(text_slot, value)
        } else {
            let vector_slot = renderer
                .get_vector_slot(rule_id)
                .ok_or_else(|| format!("vector slot '{}' not found", rule_id))?;

            self.apply_to_vector(vector_slot, value)
        }
    }

    pub fn apply_to_text(&self, slot: &mut TextSlot, value: &[f32]) -> Result<(), String> {
        let xy_value = Self::normalize_xy(value);

        // TextSlot internally always uses keyframes vec,
        // where a "static" value is represented as a single keyframe at frame 0
        match self {
            VectorPath::WrapSize => {
                // Static path targets first keyframe (frame 0)
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.wrap_size = Some(xy_value);
                Ok(())
            }
            VectorPath::WrapPosition => {
                // Static path targets first keyframe (frame 0)
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.wrap_position = Some(xy_value);
                Ok(())
            }
            VectorPath::KeyframeWrapSize(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.wrap_size = Some(xy_value);
                Ok(())
            }
            VectorPath::KeyframeWrapPosition(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.wrap_position = Some(xy_value);
                Ok(())
            }
            VectorPath::StaticValue | VectorPath::Keyframe(_) => {
                Err("path targets vector/position slot, not text slot".to_string())
            }
        }
    }

    pub fn apply_to_vector(&self, slot: &mut VectorSlot, value: &[f32]) -> Result<(), String> {
        let xy_value = Self::normalize_xy(value);

        match self {
            VectorPath::StaticValue => {
                slot.value = PropertyValue::Static(xy_value);
                Ok(())
            }
            VectorPath::Keyframe(keyframe) => match &mut slot.value {
                PropertyValue::Animated(keyframes) => {
                    let kf = keyframes
                        .get_mut(*keyframe)
                        .ok_or_else(|| format!("index {keyframe} out of bounds"))?;
                    kf.start_value = xy_value;
                    Ok(())
                }
                PropertyValue::Static(_) => Err("Vector binding path was for animated slot property, apply_to_vector was supplied an animated Vector slot.".to_string()),
            },
            _ => Err("path targets text slot, not vector/position slot".to_string()),
        }
    }

    /// Normalize input to [x, y] format
    fn normalize_xy(value: &[f32]) -> [f32; 2] {
        match value.len() {
            0 => [0.0, 0.0],
            1 => [value[0], 0.0],
            _ => [value[0], value[1]],
        }
    }

    /// Returns true if this path targets a text slot
    pub fn targets_text(&self) -> bool {
        matches!(
            self,
            VectorPath::WrapSize
                | VectorPath::WrapPosition
                | VectorPath::KeyframeWrapSize(_)
                | VectorPath::KeyframeWrapPosition(_)
        )
    }
}
