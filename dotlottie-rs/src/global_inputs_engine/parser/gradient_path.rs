use crate::{
    slots::{GradientStop, GradientValue},
    GradientSlot, LottieRenderer,
};

#[derive(Debug, Clone, Default)]
pub enum GradientPath {
    // Gradient -> Gradient target
    #[default]
    StaticValue, // value
    Keyframe(usize), // keyframes/{kf}/value
}

impl GradientPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            // Gradient -> Gradient (full replacement)
            ["value"] => Ok(GradientPath::StaticValue),
            ["keyframes", kf_idx, "value"] => {
                let kf: usize = kf_idx
                    .parse()
                    .map_err(|_| format!("invalid keyframe index: {kf_idx}"))?;
                Ok(GradientPath::Keyframe(kf))
            }

            _ => Err(format!("invalid gradient path: {path}")),
        }
    }

    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: &[GradientStop],
    ) -> Result<(), String> {
        let gradient_slot = renderer
            .get_gradient_slot(rule_id)
            .ok_or_else(|| format!("gradient slot '{rule_id}' not found"))?;
        self.apply_to_gradient(gradient_slot, value)
    }

    pub fn apply_to_gradient(
        &self,
        slot: &mut GradientSlot,
        value: &[GradientStop],
    ) -> Result<(), String> {
        match self {
            GradientPath::StaticValue => {
                let gradient_data = Self::stops_to_gradient_data(value, slot.num_stops);
                slot.data.value = GradientValue::Static(gradient_data);
                Ok(())
            }
            GradientPath::Keyframe(kf_idx) => match &mut slot.data.value {
                GradientValue::Animated(keyframes) => {
                    let kf = keyframes
                        .get_mut(*kf_idx)
                        .ok_or_else(|| format!("keyframe index {kf_idx} out of bounds"))?;
                    kf.start_value = Self::stops_to_gradient_data(value, slot.num_stops);
                    Ok(())
                }
                GradientValue::Static(_) => Err(
                    "Gradient binding path was for animated slot, but slot is static".to_string(),
                ),
            },
        }
    }

    /// Convert gradient stops to the internal gradient data format
    /// Format: [offset, r, g, b, offset, r, g, b, ...] + [offset, a, offset, a, ...]
    fn stops_to_gradient_data(stops: &[GradientStop], num_stops: usize) -> Vec<f32> {
        let actual_stops = if num_stops > 0 {
            num_stops
        } else {
            stops.len()
        };
        let mut data = Vec::with_capacity(actual_stops * 6); // 4 for color + 2 for alpha per stop

        // Color data: [offset, r, g, b] per stop
        for stop in stops.iter().take(actual_stops) {
            data.push(stop.offset);
            if stop.color.len() >= 3 {
                data.push(stop.color[0]);
                data.push(stop.color[1]);
                data.push(stop.color[2]);
            } else {
                data.push(0.0);
                data.push(0.0);
                data.push(0.0);
            }
        }

        // Pad with empty stops if needed
        for i in stops.len()..actual_stops {
            let offset = if actual_stops > 1 {
                i as f32 / (actual_stops - 1) as f32
            } else {
                0.0
            };
            data.push(offset);
            data.push(0.0);
            data.push(0.0);
            data.push(0.0);
        }

        // Alpha data: [offset, a] per stop
        for stop in stops.iter().take(actual_stops) {
            data.push(stop.offset);
            if stop.color.len() >= 4 {
                data.push(stop.color[3]);
            } else {
                data.push(1.0);
            }
        }

        // Pad alpha with defaults if needed
        for i in stops.len()..actual_stops {
            let offset = if actual_stops > 1 {
                i as f32 / (actual_stops - 1) as f32
            } else {
                0.0
            };
            data.push(offset);
            data.push(1.0);
        }

        data
    }
}
