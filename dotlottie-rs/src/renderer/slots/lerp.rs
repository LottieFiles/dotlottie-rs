use super::{ColorValue, GradientSlot, LottieProperty, PropertyValue, ScalarValue, SlotType};

#[inline]
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Interpolates `from` toward `to` at eased progress `t`, or returns `None` if this
/// pair can't be smoothly interpolated (mismatched variants other than Vector/Position,
/// either side genuinely animated — i.e. two or more keyframes — either side has an
/// expression, or gradients with a mismatched stop layout) — callers should apply `to`
/// immediately in that case instead of animating toward it. A single-keyframe "animated"
/// property carries no actual motion, so it's treated as a constant for lerp purposes (see
/// `as_constant`). Vector and Position are the same underlying `[f32; 2]` representation
/// (they only differ in whether spatial tangents are typically used), and a slot's inferred
/// type commonly disagrees with a theme's explicit type for the same numeric data — so the
/// two are lerp-compatible with each other; the result always takes on `to`'s variant.
pub(crate) fn lerp_slot(from: &SlotType, to: &SlotType, t: f32) -> Option<SlotType> {
    match (from, to) {
        (SlotType::Color(a), SlotType::Color(b)) => {
            lerp_static(a, b, t, |a: &ColorValue, b: &ColorValue, t| {
                ColorValue([
                    lerp_f32(a.0[0], b.0[0], t).clamp(0.0, 1.0),
                    lerp_f32(a.0[1], b.0[1], t).clamp(0.0, 1.0),
                    lerp_f32(a.0[2], b.0[2], t).clamp(0.0, 1.0),
                ])
            })
            .map(SlotType::Color)
        }
        (SlotType::Scalar(a), SlotType::Scalar(b)) => {
            lerp_static(a, b, t, |a: &ScalarValue, b: &ScalarValue, t| {
                ScalarValue(lerp_f32(a.0, b.0, t))
            })
            .map(SlotType::Scalar)
        }
        (SlotType::Vector(a), SlotType::Vector(b)) => {
            lerp_static(a, b, t, lerp_vec2).map(SlotType::Vector)
        }
        (SlotType::Position(a), SlotType::Position(b)) => {
            lerp_static(a, b, t, lerp_vec2).map(SlotType::Position)
        }
        (SlotType::Vector(a), SlotType::Position(b)) => {
            lerp_static(a, b, t, lerp_vec2).map(SlotType::Position)
        }
        (SlotType::Position(a), SlotType::Vector(b)) => {
            lerp_static(a, b, t, lerp_vec2).map(SlotType::Vector)
        }
        (SlotType::Gradient(a), SlotType::Gradient(b)) => lerp_gradient(a, b, t),
        _ => None,
    }
}

#[inline]
fn lerp_vec2(a: &[f32; 2], b: &[f32; 2], t: f32) -> [f32; 2] {
    [lerp_f32(a[0], b[0], t), lerp_f32(a[1], b[1], t)]
}

/// Returns the constant value behind a property, if it has one: either a `Static` value, or
/// an `Animated` property with exactly one keyframe (which carries no actual motion — it's
/// just a constant expressed as a keyframe, a common theme-authoring pattern). Two or more
/// keyframes means it's genuinely animated, so `None` is returned.
fn as_constant<T>(value: &PropertyValue<T>) -> Option<&T> {
    match value {
        PropertyValue::Static(v) => Some(v),
        PropertyValue::Animated(kfs) => match kfs.as_slice() {
            [kf] => Some(&kf.start_value),
            _ => None,
        },
    }
}

fn lerp_static<T: Clone>(
    from: &LottieProperty<T>,
    to: &LottieProperty<T>,
    t: f32,
    f: impl Fn(&T, &T, f32) -> T,
) -> Option<LottieProperty<T>> {
    if from.expression.is_some() || to.expression.is_some() {
        return None;
    }
    let a = as_constant(&from.value)?;
    let b = as_constant(&to.value)?;
    Some(LottieProperty::static_value(f(a, b, t)))
}

fn lerp_gradient(from: &GradientSlot, to: &GradientSlot, t: f32) -> Option<SlotType> {
    if from.expression.is_some() || to.expression.is_some() {
        return None;
    }
    if from.num_stops == 0 || from.num_stops != to.num_stops {
        return None;
    }
    if from.data.expression.is_some() || to.data.expression.is_some() {
        return None;
    }
    let a = as_constant(&from.data.value)?;
    let b = as_constant(&to.data.value)?;
    if a.len() != b.len() {
        return None;
    }
    let lerped: Vec<f32> = a
        .iter()
        .zip(b)
        .map(|(x, y)| lerp_f32(*x, *y, t).clamp(0.0, 1.0))
        .collect();
    Some(SlotType::Gradient(GradientSlot {
        data: LottieProperty::static_value(lerped),
        num_stops: to.num_stops,
        expression: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::slots::{
        ColorSlot, GradientStop, LottieKeyframe, PositionSlot, ScalarSlot, VectorSlot,
    };

    #[test]
    fn color_lerps_componentwise() {
        let from = SlotType::Color(ColorSlot::new([0.0, 0.0, 0.0]));
        let to = SlotType::Color(ColorSlot::new([1.0, 0.5, 0.25]));

        let at0 = lerp_slot(&from, &to, 0.0).unwrap();
        let SlotType::Color(c) = &at0 else { panic!() };
        assert!(
            matches!(&c.value, PropertyValue::Static(ColorValue([r,g,b])) if *r==0.0 && *g==0.0 && *b==0.0)
        );

        let at1 = lerp_slot(&from, &to, 1.0).unwrap();
        let SlotType::Color(c) = &at1 else { panic!() };
        assert!(
            matches!(&c.value, PropertyValue::Static(ColorValue([r,g,b])) if *r==1.0 && *g==0.5 && *b==0.25)
        );

        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Color(c) = &mid else { panic!() };
        assert!(
            matches!(&c.value, PropertyValue::Static(ColorValue([r,g,b])) if (*r-0.5).abs()<1e-6 && (*g-0.25).abs()<1e-6 && (*b-0.125).abs()<1e-6)
        );
    }

    #[test]
    fn color_clamps_overshoot_progress() {
        let from = SlotType::Color(ColorSlot::new([0.0, 0.0, 0.0]));
        let to = SlotType::Color(ColorSlot::new([1.0, 1.0, 1.0]));
        let over = lerp_slot(&from, &to, 1.5).unwrap();
        let SlotType::Color(c) = &over else { panic!() };
        assert!(
            matches!(&c.value, PropertyValue::Static(ColorValue([r,g,b])) if *r==1.0 && *g==1.0 && *b==1.0)
        );
    }

    #[test]
    fn scalar_lerps() {
        let from = SlotType::Scalar(ScalarSlot::new(0.0));
        let to = SlotType::Scalar(ScalarSlot::new(10.0));
        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Scalar(s) = &mid else { panic!() };
        assert!(matches!(&s.value, PropertyValue::Static(ScalarValue(v)) if (*v-5.0).abs()<1e-6));
    }

    #[test]
    fn vector_lerps_componentwise() {
        let from = SlotType::Vector(VectorSlot::static_value([0.0, 0.0]));
        let to = SlotType::Vector(VectorSlot::static_value([10.0, 20.0]));
        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Vector(v) = &mid else { panic!() };
        assert!(
            matches!(&v.value, PropertyValue::Static([x,y]) if (*x-5.0).abs()<1e-6 && (*y-10.0).abs()<1e-6)
        );
    }

    #[test]
    fn position_lerps_componentwise() {
        let from = SlotType::Position(PositionSlot::static_value([0.0, 0.0]));
        let to = SlotType::Position(PositionSlot::static_value([4.0, 8.0]));
        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Position(p) = &mid else {
            panic!()
        };
        assert!(
            matches!(&p.value, PropertyValue::Static([x,y]) if (*x-2.0).abs()<1e-6 && (*y-4.0).abs()<1e-6)
        );
    }

    #[test]
    fn gradient_lerps_matching_stops() {
        let from = SlotType::Gradient(GradientSlot::new(vec![
            GradientStop {
                offset: 0.0,
                color: [0.0, 0.0, 0.0, 1.0],
            },
            GradientStop {
                offset: 1.0,
                color: [0.0, 0.0, 0.0, 1.0],
            },
        ]));
        let to = SlotType::Gradient(GradientSlot::new(vec![
            GradientStop {
                offset: 0.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            GradientStop {
                offset: 1.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]));
        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Gradient(g) = &mid else {
            panic!()
        };
        let PropertyValue::Static(data) = &g.data.value else {
            panic!()
        };
        assert!(data
            .iter()
            .all(|v| (*v - 0.5).abs() < 1e-6 || *v == 0.0 || *v == 1.0));
    }

    #[test]
    fn gradient_mismatched_stop_count_is_none() {
        let from = SlotType::Gradient(GradientSlot::new(vec![GradientStop {
            offset: 0.0,
            color: [0.0, 0.0, 0.0, 1.0],
        }]));
        let to = SlotType::Gradient(GradientSlot::new(vec![
            GradientStop {
                offset: 0.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            GradientStop {
                offset: 1.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]));
        assert!(lerp_slot(&from, &to, 0.5).is_none());
    }

    #[test]
    fn type_mismatch_is_none() {
        let from = SlotType::Color(ColorSlot::new([0.0, 0.0, 0.0]));
        let to = SlotType::Scalar(ScalarSlot::new(1.0));
        assert!(lerp_slot(&from, &to, 0.5).is_none());
    }

    #[test]
    fn vector_and_position_lerp_across_each_other_taking_tos_variant() {
        // A slot's inferred type (from raw JSON with no explicit type tag) commonly disagrees
        // with a theme's explicit Position/Vector type for the same [f32;2] data.
        let from = SlotType::Vector(VectorSlot::static_value([0.0, 0.0]));
        let to = SlotType::Position(PositionSlot::static_value([10.0, 20.0]));

        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Position(p) = &mid else {
            panic!("result should take on `to`'s variant (Position)")
        };
        assert!(
            matches!(&p.value, PropertyValue::Static([x, y]) if (*x - 5.0).abs() < 1e-6 && (*y - 10.0).abs() < 1e-6)
        );

        // And the reverse direction.
        let from = SlotType::Position(PositionSlot::static_value([0.0, 0.0]));
        let to = SlotType::Vector(VectorSlot::static_value([10.0, 20.0]));
        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        assert!(matches!(&mid, SlotType::Vector(_)));
    }

    #[test]
    fn zero_keyframe_property_is_none() {
        let from = SlotType::Color(ColorSlot::new([0.0, 0.0, 0.0]));
        let to = SlotType::Color(ColorSlot::with_keyframes(vec![]));
        assert!(lerp_slot(&from, &to, 0.5).is_none());
    }

    #[test]
    fn multi_keyframe_property_is_none() {
        let from = SlotType::Color(ColorSlot::new([0.0, 0.0, 0.0]));
        let to = SlotType::Color(ColorSlot::with_keyframes(vec![
            LottieKeyframe {
                frame: 0,
                start_value: ColorValue([1.0, 1.0, 1.0]),
                in_tangent: None,
                out_tangent: None,
                value_in_tangent: None,
                value_out_tangent: None,
                hold: None,
            },
            LottieKeyframe {
                frame: 30,
                start_value: ColorValue([0.0, 0.0, 0.0]),
                in_tangent: None,
                out_tangent: None,
                value_in_tangent: None,
                value_out_tangent: None,
                hold: None,
            },
        ]));
        assert!(lerp_slot(&from, &to, 0.5).is_none());
    }

    #[test]
    fn single_keyframe_property_lerps_as_a_constant() {
        // A common theme-authoring pattern: a "static" value expressed as a single keyframe
        // rather than the `value` field. It carries no real animation, so it should lerp.
        let from = SlotType::Position(PositionSlot::static_value([0.0, 0.0]));
        let to = SlotType::Position(LottieProperty::animated(vec![LottieKeyframe {
            frame: 0,
            start_value: [10.0, 20.0],
            in_tangent: None,
            out_tangent: None,
            value_in_tangent: None,
            value_out_tangent: None,
            hold: None,
        }]));

        let mid = lerp_slot(&from, &to, 0.5).unwrap();
        let SlotType::Position(p) = &mid else {
            panic!()
        };
        assert!(
            matches!(&p.value, PropertyValue::Static([x, y]) if (*x - 5.0).abs() < 1e-6 && (*y - 10.0).abs() < 1e-6)
        );
    }

    #[test]
    fn expression_is_none() {
        let from = SlotType::Color(ColorSlot::new([0.0, 0.0, 0.0]));
        let to = SlotType::Color(ColorSlot::new([1.0, 1.0, 1.0]).with_expression("x".to_string()));
        assert!(lerp_slot(&from, &to, 0.5).is_none());
    }
}
