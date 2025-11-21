use super::LottieProperty;

/// 2D position slot for properties with spatial bezier curves.
/// Note: PositionSlot and VectorSlot are type-identical, both are LottieProperty<[f32; 2]>.
/// The semantic difference is that Position properties typically use spatial tangents (ti/to)
/// in their keyframes for curved motion paths, while Vector properties typically don't.
///
/// Use `LottieProperty::static_value([x, y])` or `LottieProperty::animated(keyframes)` to construct.
pub type PositionSlot = LottieProperty<[f32; 2]>;
