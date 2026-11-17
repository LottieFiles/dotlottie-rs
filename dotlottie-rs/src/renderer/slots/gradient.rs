use super::{property_from_json, write_property, LottieKeyframe, LottieProperty};
use crate::json::{write_str, ObjWriter, Value};
use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub struct GradientStop {
    pub offset: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct GradientSlot {
    pub data: LottieProperty<Vec<f32>>,
    pub num_stops: usize,
    pub expression: Option<String>,
}

impl GradientSlot {
    pub fn new(stops: Vec<GradientStop>) -> Self {
        let num_stops = stops.len();
        let gradient_data = Self::stops_to_lottie_data(&stops);

        Self {
            data: LottieProperty::static_value(gradient_data),
            num_stops,
            expression: None,
        }
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<Vec<GradientStop>>>) -> Self {
        let num_stops = keyframes.first().map_or(0, |kf| kf.start_value.len());

        let lottie_keyframes = keyframes
            .into_iter()
            .map(|kf| LottieKeyframe {
                frame: kf.frame,
                start_value: Self::stops_to_lottie_data(&kf.start_value),
                in_tangent: kf.in_tangent,
                out_tangent: kf.out_tangent,
                value_in_tangent: kf.value_in_tangent,
                value_out_tangent: kf.value_out_tangent,
                hold: kf.hold,
            })
            .collect();

        Self {
            data: LottieProperty::animated(lottie_keyframes),
            num_stops,
            expression: None,
        }
    }

    pub fn with_expression(mut self, expr: String) -> Self {
        self.expression = Some(expr);
        self
    }

    fn stops_to_lottie_data(stops: &[GradientStop]) -> Vec<f32> {
        let mut data = Vec::with_capacity(stops.len() * 6);
        for s in stops {
            data.extend_from_slice(&[s.offset, s.color[0], s.color[1], s.color[2]]);
        }
        for s in stops {
            data.extend_from_slice(&[s.offset, s.color[3]]);
        }
        data
    }
}

pub(crate) fn gradient_slot_from_json(v: &Value) -> Option<GradientSlot> {
    Some(GradientSlot {
        data: property_from_json(v.get("k")?)?,
        num_stops: v.u32_field("p")? as usize,
        expression: v.opt_str_field("x")?,
    })
}

pub(crate) fn write_gradient_slot(g: &GradientSlot, out: &mut String) {
    let mut o = ObjWriter::new(out);
    write_property(&g.data, o.field("k"));
    let _ = write!(o.field("p"), "{}", g.num_stops);
    if let Some(x) = &g.expression {
        write_str(x, o.field("x"));
    }
    o.finish();
}
