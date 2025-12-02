use crate::{LottieRenderer, TextSlot};

#[derive(Debug, Clone, Default)]
pub enum BooleanPath {
    #[default]
    StaticValue,

    // Boolean -> TextDocument targets
    StrokeOverFill,                // value/strokeOverFill
    KeyframeStrokeOverFill(usize), // keyframes/{kf}/value/strokeOverFill
}

impl BooleanPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            // Boolean static value (for future boolean slots if needed)
            ["value"] => Ok(BooleanPath::StaticValue),

            // Boolean -> TextDocument (static)
            ["value", "strokeOverFill"] => Ok(BooleanPath::StrokeOverFill),

            // Boolean -> TextDocument (animated)
            ["keyframes", kf_idx, "value", "strokeOverFill"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(BooleanPath::KeyframeStrokeOverFill(kf))
            }

            _ => Err(format!("invalid boolean path: {path}")),
        }
    }

    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: bool,
    ) -> Result<(), String> {
        if self.targets_text() {
            let text_slot = renderer
                .get_text_slot(rule_id)
                .ok_or_else(|| format!("text slot '{}' not found", rule_id))?;
            self.apply_to_text(text_slot, value)
        } else {
            // For future: could have boolean slots
            Err("boolean slot not yet implemented".to_string())
        }
    }

    pub fn apply_to_text(&self, slot: &mut TextSlot, value: bool) -> Result<(), String> {
        match self {
            BooleanPath::StrokeOverFill => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.stroke_over_fill = Some(value);
                Ok(())
            }
            BooleanPath::KeyframeStrokeOverFill(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.stroke_over_fill = Some(value);
                Ok(())
            }
            BooleanPath::StaticValue => {
                Err("static boolean value not supported for text slot".to_string())
            }
        }
    }

    pub fn targets_text(&self) -> bool {
        matches!(
            self,
            BooleanPath::StrokeOverFill | BooleanPath::KeyframeStrokeOverFill(_)
        )
    }
}
