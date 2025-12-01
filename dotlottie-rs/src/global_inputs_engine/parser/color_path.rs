use crate::{
    slots::{GradientValue, PropertyValue},
    ColorSlot, GradientSlot,
};

#[derive(Debug, Clone, Default)]
pub enum ColorPath {
    #[default]
    StaticValue,
    Keyframe(usize),

    // Color -> Gradient targets
    GradientStop(usize),                // value/{stop}/color
    GradientKeyframeStop(usize, usize), // keyframes/{kf}/value/{stop}/color
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

            _ => Err(format!("invalid path: {path}")),
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
}
