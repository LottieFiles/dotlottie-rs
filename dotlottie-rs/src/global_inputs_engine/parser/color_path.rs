use crate::{
    slots::{GradientValue, PropertyValue},
    ColorSlot, GradientSlot, LottieRenderer, TextSlot,
};

#[derive(Debug, Clone, Default)]
pub enum ColorPath {
    #[default]
    StaticValue,
    Keyframe(usize),

    // Color -> Gradient targets
    GradientStop(usize),                // value/{stop}/color
    GradientKeyframeStop(usize, usize), // keyframes/{kf}/value/{stop}/color

    // Color -> Text targets
    FillColor,                  // value/fillColor
    StrokeColor,                // value/strokeColor
    KeyframeFillColor(usize),   // keyframes/{kf}/value/fillColor
    KeyframeStrokeColor(usize), // keyframes/{kf}/value/strokeColor
}

impl ColorPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let parts: Vec<&str> = path.split('/').collect();

        println!("Parts: {:?}", parts);

        match parts.as_slice() {
            // Color -> Color
            ["value"] => Ok(ColorPath::StaticValue),
            ["keyframes", idx, "value"] => {
                let i: usize = idx.parse().map_err(|_| format!("invalid index: {idx}"))?;
                Ok(ColorPath::Keyframe(i))
            }

            // Color -> Gradient (static)
            ["value", stop_idx, "color"] => {
                let stop: usize = stop_idx
                    .parse()
                    .map_err(|_| format!("invalid stop index: {stop_idx}"))?;
                Ok(ColorPath::GradientStop(stop))
            }

            // Color -> Gradient (animated)
            ["keyframes", kf_idx, "value", stop_idx, "color"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                let stop: usize = stop_idx
                    .parse()
                    .map_err(|_| format!("invalid stop index: {stop_idx}"))?;
                Ok(ColorPath::GradientKeyframeStop(kf, stop))
            }

            // Color -> Text (static)
            ["value", "fillColor"] => Ok(ColorPath::FillColor),
            ["value", "strokeColor"] => Ok(ColorPath::StrokeColor),

            // Color -> Text (animated)
            ["keyframes", kf_idx, "value", "fillColor"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(ColorPath::KeyframeFillColor(kf))
            }
            ["keyframes", kf_idx, "value", "strokeColor"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(ColorPath::KeyframeStrokeColor(kf))
            }

            _ => Err(format!("invalid path: {path}")),
        }
    }

    /// Apply this color path to the appropriate slot type
    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: &Vec<f32>,
    ) -> Result<(), String> {
        if self.targets_gradient() {
            let gradient_slot = renderer
                .get_gradient_slot(rule_id)
                .ok_or_else(|| format!("gradient slot '{}' not found", rule_id))?;
            self.apply_to_gradient(gradient_slot, value)
        } else if self.targets_text() {
            let text_slot = renderer
                .get_text_slot(rule_id)
                .ok_or_else(|| format!("text slot '{}' not found", rule_id))?;
            self.apply_to_text(text_slot, value)
        } else {
            let color_slot = renderer
                .get_color_slot(rule_id)
                .ok_or_else(|| format!("color slot '{}' not found", rule_id))?;
            self.apply_to_color(color_slot, value)
        }
    }

    pub fn apply_to_color(&self, slot: &mut ColorSlot, value: &Vec<f32>) -> Result<(), String> {
        let rgba_value = if value.len() >= 4 {
            [value[0], value[1], value[2], value[3]]
        } else if value.len() >= 3 {
            [value[0], value[1], value[2], 1.0]
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };

        match self {
            ColorPath::StaticValue => {
                slot.value = PropertyValue::Static(rgba_value);
                Ok(())
            }
            ColorPath::Keyframe(i) => match &mut slot.value {
                PropertyValue::Animated(keyframes) => {
                    let kf = keyframes
                        .get_mut(*i)
                        .ok_or_else(|| format!("index {i} out of bounds"))?;
                    kf.start_value = rgba_value;
                    println!("KF: {:?}", kf);
                    Ok(())
                }
                PropertyValue::Static(_) => Err("slot is not animated".to_string()),
            },
            ColorPath::GradientStop(_) | ColorPath::GradientKeyframeStop(_, _) => {
                Err("path targets gradient, not color slot".to_string())
            }
            ColorPath::FillColor
            | ColorPath::StrokeColor
            | ColorPath::KeyframeFillColor(_)
            | ColorPath::KeyframeStrokeColor(_) => {
                Err("path targets text slot, not color slot".to_string())
            }
        }
    }

    pub fn apply_to_gradient(
        &self,
        slot: &mut GradientSlot,
        value: &Vec<f32>,
    ) -> Result<(), String> {
        let num_stops = slot.num_stops;
        let rgba_value = if value.len() >= 4 {
            [value[0], value[1], value[2], value[3]]
        } else if value.len() >= 3 {
            [value[0], value[1], value[2], 1.0]
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };

        match self {
            ColorPath::GradientStop(stop_idx) => match &mut slot.data.value {
                GradientValue::Static(data) => {
                    Self::set_stop_color(data, *stop_idx, num_stops, rgba_value)
                }
                GradientValue::Animated(_) => {
                    Err("expected static gradient, got animated".to_string())
                }
            },
            ColorPath::GradientKeyframeStop(kf_idx, stop_idx) => match &mut slot.data.value {
                GradientValue::Animated(keyframes) => {
                    let kf = keyframes
                        .get_mut(*kf_idx)
                        .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                    Self::set_stop_color(&mut kf.start_value, *stop_idx, num_stops, rgba_value)
                }
                GradientValue::Static(_) => {
                    Err("expected animated gradient, got static".to_string())
                }
            },
            ColorPath::StaticValue | ColorPath::Keyframe(_) => {
                Err("path targets color slot, not gradient".to_string())
            }
            ColorPath::FillColor
            | ColorPath::StrokeColor
            | ColorPath::KeyframeFillColor(_)
            | ColorPath::KeyframeStrokeColor(_) => {
                Err("path targets text slot, not gradient slot".to_string())
            }
        }
    }

    pub fn apply_to_text(&self, slot: &mut TextSlot, value: &Vec<f32>) -> Result<(), String> {
        let color_value = if value.len() >= 3 {
            value.clone()
        } else {
            vec![0.0, 0.0, 0.0]
        };

        match self {
            ColorPath::FillColor => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.fill_color = Some(color_value);
                Ok(())
            }
            ColorPath::StrokeColor => {
                let kf = slot
                    .keyframes
                    .first_mut()
                    .ok_or_else(|| "text slot has no keyframes".to_string())?;
                kf.text_document.stroke_color = Some(color_value);
                Ok(())
            }
            ColorPath::KeyframeFillColor(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.fill_color = Some(color_value);
                Ok(())
            }
            ColorPath::KeyframeStrokeColor(kf_idx) => {
                let kf = slot
                    .keyframes
                    .get_mut(*kf_idx)
                    .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                kf.text_document.stroke_color = Some(color_value);
                Ok(())
            }
            ColorPath::StaticValue | ColorPath::Keyframe(_) => {
                Err("path targets color slot, not text slot".to_string())
            }
            ColorPath::GradientStop(_) | ColorPath::GradientKeyframeStop(_, _) => {
                Err("path targets gradient slot, not text slot".to_string())
            }
        }
    }

    fn set_stop_color(
        data: &mut Vec<f32>,
        stop_idx: usize,
        num_stops: usize,
        value: [f32; 4],
    ) -> Result<(), String> {
        if stop_idx >= num_stops {
            return Err(format!(
                "stop index {stop_idx} out of bounds (num_stops: {num_stops})"
            ));
        }

        // Color data: [offset, r, g, b] per stop
        let color_base = stop_idx * 4;

        // Alpha data starts after all color data: [offset, alpha] per stop
        let alpha_base = num_stops * 4 + stop_idx * 2;

        // Check we have enough data for color
        if color_base + 3 >= data.len() {
            return Err(format!(
                "gradient data too short for color at stop {stop_idx} (need index {}, len {})",
                color_base + 3,
                data.len()
            ));
        }

        // Set RGB (keep offset at color_base)
        data[color_base + 1] = value[0];
        data[color_base + 2] = value[1];
        data[color_base + 3] = value[2];

        // Set alpha if data is long enough
        if alpha_base + 1 < data.len() {
            data[alpha_base + 1] = value[3];
        }

        Ok(())
    }

    /// Returns true if this path targets a gradient slot
    pub fn targets_gradient(&self) -> bool {
        matches!(
            self,
            ColorPath::GradientStop(_) | ColorPath::GradientKeyframeStop(_, _)
        )
    }

    /// Returns true if this path targets a text slot
    pub fn targets_text(&self) -> bool {
        matches!(
            self,
            ColorPath::FillColor
                | ColorPath::StrokeColor
                | ColorPath::KeyframeFillColor(_)
                | ColorPath::KeyframeStrokeColor(_)
        )
    }
}
