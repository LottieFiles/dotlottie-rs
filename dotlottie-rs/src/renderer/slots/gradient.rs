use super::{Bezier, LottieKeyframe};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientProperty {
    #[serde(rename = "a")]
    pub animated: u8,
    #[serde(rename = "k")]
    pub value: GradientValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GradientValue {
    Static(Vec<f32>),
    Animated(Vec<GradientKeyframe>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientKeyframe {
    #[serde(rename = "t")]
    pub frame: u32,
    #[serde(rename = "s")]
    pub start_value: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "i")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "o")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "h")]
    pub hold: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientSlot {
    #[serde(rename = "k")]
    pub data: GradientProperty,
    #[serde(rename = "p")]
    pub num_stops: usize,
    #[serde(skip_serializing_if = "Option::is_none", rename = "x")]
    pub expression: Option<String>,
}

impl GradientSlot {
    pub fn new(stops: Vec<GradientStop>) -> Self {
        let num_stops = stops.len();
        let gradient_data = Self::stops_to_lottie_data(&stops);

        Self {
            data: GradientProperty {
                animated: 0,
                value: GradientValue::Static(gradient_data),
            },
            num_stops,
            expression: None,
        }
    }

    pub fn with_keyframes(keyframes: Vec<LottieKeyframe<Vec<GradientStop>>>) -> Self {
        let num_stops = keyframes
            .first()
            .map(|kf| kf.start_value.len())
            .unwrap_or(0);

        let lottie_keyframes: Vec<GradientKeyframe> = keyframes
            .into_iter()
            .map(|kf| GradientKeyframe {
                frame: kf.frame,
                start_value: Self::stops_to_lottie_data(&kf.start_value),
                in_tangent: kf.in_tangent,
                out_tangent: kf.out_tangent,
                hold: kf.hold,
            })
            .collect();

        Self {
            data: GradientProperty {
                animated: 1,
                value: GradientValue::Animated(lottie_keyframes),
            },
            num_stops,
            expression: None,
        }
    }

    pub fn with_expression(mut self, expr: String) -> Self {
        self.expression = Some(expr);
        self
    }

    fn stops_to_lottie_data(stops: &[GradientStop]) -> Vec<f32> {
        let mut gradient_data = vec![];
        let mut transparency_data = vec![];
        let alpha_present = stops.iter().any(|stop| stop.color.len() == 4);

        for stop in stops {
            gradient_data.push(stop.offset);
            gradient_data.push(stop.color[0]);
            gradient_data.push(stop.color[1]);
            gradient_data.push(stop.color[2]);

            if alpha_present {
                let alpha = if stop.color.len() == 4 {
                    stop.color[3]
                } else {
                    1.0
                };
                transparency_data.push(stop.offset);
                transparency_data.push(alpha);
            }
        }

        gradient_data.extend(transparency_data);
        gradient_data
    }
}
