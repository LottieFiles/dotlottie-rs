use super::LottieProperty;

/// 2D vector slot for properties like scale, size, etc.
/// Note: VectorSlot and PositionSlot are type-identical, both are LottieProperty<[f32; 2]>.
/// The semantic difference is that Position properties typically use spatial tangents (ti/to)
/// while Vector properties typically don't.
///
/// Use `LottieProperty::static_value([x, y])` or `LottieProperty::animated(keyframes)` to construct.
pub type VectorSlot = LottieProperty<[f32; 2]>;
