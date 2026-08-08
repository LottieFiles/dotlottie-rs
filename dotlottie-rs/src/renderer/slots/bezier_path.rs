use super::{write_f32_slice, LottieProperty, PropertyValue, SlotValue};
use crate::json::{array_of, f32_array, write_seq, ObjWriter, Value};
use std::fmt::Write as _;

/// A single bezier path value: the `{i, o, v, c}` object Lottie uses for path
/// (`ty: "sh"`) data. Tangents are relative to the vertex at the same index.
#[derive(Debug, Clone, PartialEq)]
pub struct BezierPath {
    pub in_tangents: Vec<[f32; 2]>,
    pub out_tangents: Vec<[f32; 2]>,
    pub vertices: Vec<[f32; 2]>,
    pub closed: bool,
}

/// Bezier path slot, matching ThorVG's `LottieProperty::Type::PathSet`.
pub type BezierPathSlot = LottieProperty<BezierPath>;

impl BezierPath {
    /// Straight-line path through `vertices` (all tangents zero).
    pub fn polygon(vertices: Vec<[f32; 2]>, closed: bool) -> Self {
        Self {
            in_tangents: vec![[0.0, 0.0]; vertices.len()],
            out_tangents: vec![[0.0, 0.0]; vertices.len()],
            vertices,
            closed,
        }
    }

    pub fn is_valid(&self) -> bool {
        let finite = |pts: &[[f32; 2]]| pts.iter().flatten().all(|c| c.is_finite());
        !self.vertices.is_empty()
            && self.in_tangents.len() == self.vertices.len()
            && self.out_tangents.len() == self.vertices.len()
            && finite(&self.vertices)
            && finite(&self.in_tangents)
            && finite(&self.out_tangents)
    }
}

impl BezierPathSlot {
    pub fn new(path: BezierPath) -> Self {
        Self::static_value(path)
    }

    pub fn is_valid(&self) -> bool {
        match &self.value {
            PropertyValue::Static(p) => p.is_valid(),
            PropertyValue::Animated(kfs) => {
                !kfs.is_empty() && kfs.iter().all(|kf| kf.start_value.is_valid())
            }
        }
    }
}

/// Keyframed path values wrap the path object in a single-element array; static
/// values don't. Unwrap either shape.
fn unwrap_path_value<'v, 'a>(v: &'v Value<'a>) -> Option<&'v Value<'a>> {
    match v.as_array() {
        Some([only]) => Some(only),
        Some(_) => None,
        None => Some(v),
    }
}

pub(crate) fn is_bezier_path_value(v: &Value) -> bool {
    unwrap_path_value(v)
        .is_some_and(|o| o.get("v").is_some() && o.get("i").is_some() && o.get("o").is_some())
}

pub(crate) fn points(v: &Value) -> Option<Vec<[f32; 2]>> {
    array_of(v, f32_array::<2>)
}

fn write_points(pts: &[[f32; 2]], out: &mut String) {
    write_seq(out, pts, |p, out| write_f32_slice(p, out));
}

impl SlotValue for BezierPath {
    fn from_json(v: &Value) -> Option<Self> {
        let obj = unwrap_path_value(v)?;
        let path = BezierPath {
            in_tangents: points(obj.get("i")?)?,
            out_tangents: points(obj.get("o")?)?,
            vertices: points(obj.get("v")?)?,
            closed: obj.get("c").and_then(Value::as_bool).unwrap_or(false),
        };
        path.is_valid().then_some(path)
    }

    fn write(&self, out: &mut String) {
        let mut o = ObjWriter::new(out);
        write_points(&self.in_tangents, o.field("i"));
        write_points(&self.out_tangents, o.field("o"));
        write_points(&self.vertices, o.field("v"));
        let _ = write!(o.field("c"), "{}", self.closed);
        o.finish();
    }

    /// Lottie wraps keyframed path values in a single-element array.
    fn write_keyframe_value(&self, out: &mut String) {
        out.push('[');
        self.write(out);
        out.push(']');
    }
}
