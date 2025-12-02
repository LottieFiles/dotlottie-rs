use crate::{
    parser::{
        boolean_path::BooleanPath, color_path::ColorPath, gradient_path::GradientPath,
        numeric_path::NumericPath, string_path::StringPath, vector_path::VectorPath,
    },
    GradientStop, LottieRenderer,
};

#[derive(Debug, Clone)]
pub enum BindingPath {
    Color(ColorPath),
    Vector(VectorPath),
    Numeric(NumericPath),
    String(StringPath),
    Boolean(BooleanPath),
    Gradient(GradientPath),
    // Future: Gradient(GradientPath), Image(ImagePath), etc.
}

#[derive(Debug, Clone)]
pub enum BindingValue<'a> {
    Color(&'a [f32]),
    Vector(&'a [f32]),
    Numeric(f32),
    String(&'a str),
    Boolean(bool),
    Gradient(&'a [GradientStop]),
}

impl BindingPath {
    pub fn apply(
        &self,
        renderer: &mut Box<dyn LottieRenderer>,
        rule_id: &str,
        value: BindingValue,
    ) -> Result<(), String> {
        match self {
            BindingPath::Color(path) => {
                let color_value = match value {
                    BindingValue::Color(v) => v.to_vec(),
                    _ => return Err("expected color or vector value for color path".to_string()),
                };
                path.apply(renderer, rule_id, &color_value)
            }
            BindingPath::Vector(path) => {
                let vector_value = match value {
                    BindingValue::Vector(v) => v,
                    _ => return Err("expected vector value for vector path".to_string()),
                };
                path.apply(renderer, rule_id, vector_value)
            }
            BindingPath::Numeric(path) => {
                let numeric_value = match value {
                    BindingValue::Numeric(v) => v,
                    BindingValue::Vector(v) => v.first().copied().unwrap_or(0.0),
                    BindingValue::Color(v) => v.first().copied().unwrap_or(0.0),
                    _ => return Err("expected numeric value for numeric path".to_string()),
                };
                path.apply(renderer, rule_id, numeric_value)
            }
            BindingPath::String(path) => {
                let string_value = match value {
                    BindingValue::String(v) => v,
                    _ => return Err("expected string value for string path".to_string()),
                };
                path.apply(renderer, rule_id, string_value)
            }
            BindingPath::Boolean(path) => {
                let bool_value = match value {
                    BindingValue::Boolean(v) => v,
                    _ => return Err("expected boolean value for boolean path".to_string()),
                };
                path.apply(renderer, rule_id, bool_value)
            }
            BindingPath::Gradient(path) => {
                let gradient_value = match value {
                    BindingValue::Gradient(v) => v,
                    _ => return Err("expected gradient value for gradient path".to_string()),
                };
                path.apply(renderer, rule_id, gradient_value)
            }
        }
    }
}

impl From<ColorPath> for BindingPath {
    fn from(path: ColorPath) -> Self {
        BindingPath::Color(path)
    }
}

impl From<VectorPath> for BindingPath {
    fn from(path: VectorPath) -> Self {
        BindingPath::Vector(path)
    }
}

impl From<NumericPath> for BindingPath {
    fn from(path: NumericPath) -> Self {
        BindingPath::Numeric(path)
    }
}

impl From<StringPath> for BindingPath {
    fn from(path: StringPath) -> Self {
        BindingPath::String(path)
    }
}

impl From<BooleanPath> for BindingPath {
    fn from(path: BooleanPath) -> Self {
        BindingPath::Boolean(path)
    }
}

impl From<GradientPath> for BindingPath {
    fn from(path: GradientPath) -> Self {
        BindingPath::Gradient(path)
    }
}
