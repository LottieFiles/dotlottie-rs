use crate::{LottieRenderer, TextSlot};

#[derive(Debug, Clone, Default)]
pub enum StringPath {
    #[default]
    StaticValue,
    Keyframe(usize),

    // String -> TextDocument targets
    Text,                    // value/text
    FontName,                // value/fontName
    Justify,                 // value/justify
    TextCaps,                // value/textCaps
    KeyframeText(usize),     // keyframes/{kf}/value/text
    KeyframeFontName(usize), // keyframes/{kf}/value/fontName
    KeyframeJustify(usize),  // keyframes/{kf}/value/justify
    KeyframeTextCaps(usize), // keyframes/{kf}/value/textCaps
}

impl StringPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            // String -> Text slot (the whole text value)
            ["value"] => Ok(StringPath::StaticValue),
            ["keyframes", idx, "value"] => {
                let i: usize = idx.parse().map_err(|_| format!("invalid index: {idx}"))?;
                Ok(StringPath::Keyframe(i))
            }

            // String -> TextDocument properties (static)
            ["value", "text"] => Ok(StringPath::Text),
            ["value", "fontName"] => Ok(StringPath::FontName),
            ["value", "justify"] => Ok(StringPath::Justify),
            ["value", "textCaps"] => Ok(StringPath::TextCaps),

            // String -> TextDocument properties (animated)
            ["keyframes", kf_idx, "value", "text"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(StringPath::KeyframeText(kf))
            }
            ["keyframes", kf_idx, "value", "fontName"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(StringPath::KeyframeFontName(kf))
            }
            ["keyframes", kf_idx, "value", "justify"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(StringPath::KeyframeJustify(kf))
            }
            ["keyframes", kf_idx, "value", "textCaps"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(StringPath::KeyframeTextCaps(kf))
            }

            _ => Err(format!("invalid string path: {path}")),
        }
    }

    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: &str,
    ) -> Result<(), String> {
        // String paths always target text slots
        let text_slot = renderer
            .get_text_slot(rule_id)
            .ok_or_else(|| format!("text slot '{rule_id}' not found"))?;
        self.apply_to_text(text_slot, value)
    }

    pub fn apply_to_text(&self, slot: &mut TextSlot, value: &str) -> Result<(), String> {
        match self {
            StringPath::StaticValue | StringPath::Text => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.text = value.to_string();
                Ok(())
            }
            StringPath::FontName => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.font_name = Some(value.to_string());
                Ok(())
            }
            StringPath::Justify => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.justify = Some(Self::parse_justify(value)?);
                Ok(())
            }
            StringPath::TextCaps => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.text_caps = Some(Self::parse_text_caps(value)?);
                Ok(())
            }
            StringPath::Keyframe(kf_idx) | StringPath::KeyframeText(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.text = value.to_string();
                Ok(())
            }
            StringPath::KeyframeFontName(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.font_name = Some(value.to_string());
                Ok(())
            }
            StringPath::KeyframeJustify(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.justify = Some(Self::parse_justify(value)?);
                Ok(())
            }
            StringPath::KeyframeTextCaps(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.text_caps = Some(Self::parse_text_caps(value)?);
                Ok(())
            }
        }
    }

    /// Parse justify string to numeric value
    fn parse_justify(value: &str) -> Result<u8, String> {
        match value {
            "Left" => Ok(0),
            "Right" => Ok(1),
            "Center" => Ok(2),
            "JustifyLastLeft" => Ok(3),
            "JustifyLastRight" => Ok(4),
            "JustifyLastCenter" => Ok(5),
            "JustifyLastFull" => Ok(6),
            _ => Err(format!("invalid justify value: {value}")),
        }
    }

    /// Parse textCaps string to numeric value
    fn parse_text_caps(value: &str) -> Result<u8, String> {
        match value {
            "Regular" => Ok(0),
            "AllCaps" => Ok(1),
            "SmallCaps" => Ok(2),
            _ => Err(format!("invalid textCaps value: {value}")),
        }
    }

    pub fn targets_text(&self) -> bool {
        // All string paths target text slots
        true
    }
}
