//! Serialized motion op payloads for state machine actions and motions: node props,
//! keyframes, and transition options parsed from the SM JSON with `$input`-bindable
//! numeric leaves, resolved against engine inputs at execute time.

use crate::json::Value;
use crate::motion::{ease_preset, AnimateOptions, Prop, PropKeyframes, SpringParams, Transition};
use crate::{ClipRegion, NodeProps, SpotMask, Tint};

use super::definition::{string_number, StringNumber};
use super::{StateMachineEngine, GLOBAL_INPUT_PREFIX};

/// Resolve a numeric leaf: `"$name"` / `"@reserved"` reads an input, numbers pass
/// through. `None` when a reference doesn't resolve.
pub(crate) fn resolve_num(engine: &StateMachineEngine, value: &StringNumber) -> Option<f32> {
    match value {
        StringNumber::F32(v) => Some(*v),
        StringNumber::String(s) => {
            if s.starts_with(GLOBAL_INPUT_PREFIX) {
                engine.get_numeric_input(s)
            } else if let Some(name) = s.strip_prefix('$') {
                engine.get_numeric_input(name)
            } else {
                None
            }
        }
    }
}

fn num_field(v: &Value, key: &str) -> Option<StringNumber> {
    v.get(key).and_then(string_number)
}

fn parse_hex_rgb(s: &str) -> Option<[u8; 3]> {
    let hex = s.strip_prefix('#')?;
    if hex.len() < 6 {
        return None;
    }
    let n = u32::from_str_radix(&hex[..6], 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

fn parse_blend(s: &str) -> Option<u8> {
    Some(match s {
        "normal" => 0,
        "multiply" => 1,
        "screen" => 2,
        "overlay" => 3,
        "darken" => 4,
        "lighten" => 5,
        "colorDodge" => 6,
        "colorBurn" => 7,
        "hardLight" => 8,
        "softLight" => 9,
        "difference" => 10,
        "exclusion" => 11,
        "hue" => 12,
        "saturation" => 13,
        "color" => 14,
        "luminosity" => 15,
        "add" => 16,
        _ => return None,
    })
}

#[derive(Debug, Clone)]
struct TintSpec {
    black: [u8; 3],
    white: [u8; 3],
    intensity: StringNumber,
}

#[derive(Debug, Clone)]
struct SpotSpec {
    cx: StringNumber,
    cy: StringNumber,
    r: StringNumber,
    feather: StringNumber,
}

#[derive(Debug, Clone)]
enum ClipSpec {
    Rect {
        x: StringNumber,
        y: StringNumber,
        w: StringNumber,
        h: StringNumber,
        r: StringNumber,
    },
    Circle {
        cx: StringNumber,
        cy: StringNumber,
        r: StringNumber,
    },
}

/// `SetNode` payload: the `NodeProps` shape with `$input`-bindable numeric leaves.
/// Mirrors the wasm `parse_node_props` semantics field for field.
#[derive(Debug, Clone, Default)]
pub struct PropsSpec {
    x: Option<StringNumber>,
    y: Option<StringNumber>,
    rotate: Option<StringNumber>,
    scale: Option<StringNumber>,
    scale_x: Option<StringNumber>,
    scale_y: Option<StringNumber>,
    opacity: Option<StringNumber>,
    blur: Option<StringNumber>,
    visible: Option<bool>,
    blend: Option<u8>,
    anchor: Option<(StringNumber, StringNumber)>,
    tint: Option<TintSpec>,
    spot: Option<SpotSpec>,
    clip: Option<ClipSpec>,
}

pub(crate) fn props_spec_from_json(v: &Value) -> Option<PropsSpec> {
    let mut spec = PropsSpec {
        x: num_field(v, "x"),
        y: num_field(v, "y"),
        rotate: num_field(v, "rotate"),
        scale: num_field(v, "scale"),
        scale_x: num_field(v, "scaleX"),
        scale_y: num_field(v, "scaleY"),
        opacity: num_field(v, "opacity"),
        blur: num_field(v, "blur"),
        visible: v.get("visible").and_then(Value::as_bool),
        blend: None,
        anchor: None,
        tint: None,
        spot: None,
        clip: None,
    };
    if let Some(blend) = v.str_field("blend") {
        spec.blend = Some(parse_blend(blend)?);
    }
    if let Some(anchor) = v.get("anchor") {
        spec.anchor = Some((num_field(anchor, "x")?, num_field(anchor, "y")?));
    }
    if let Some(tint) = v.get("tint") {
        let defaults = Tint::default();
        spec.tint = Some(TintSpec {
            black: match tint.str_field("black") {
                Some(s) => parse_hex_rgb(s)?,
                None => defaults.black,
            },
            white: match tint.str_field("white") {
                Some(s) => parse_hex_rgb(s)?,
                None => defaults.white,
            },
            intensity: num_field(tint, "intensity").unwrap_or(StringNumber::F32(1.0)),
        });
    }
    if let Some(spot) = v.get("spot") {
        spec.spot = Some(SpotSpec {
            cx: num_field(spot, "cx").unwrap_or(StringNumber::F32(0.0)),
            cy: num_field(spot, "cy").unwrap_or(StringNumber::F32(0.0)),
            r: num_field(spot, "r").unwrap_or(StringNumber::F32(0.0)),
            feather: num_field(spot, "feather").unwrap_or(StringNumber::F32(0.4)),
        });
    }
    if let Some(clip) = v.get("clip") {
        let zero = || StringNumber::F32(0.0);
        spec.clip = Some(match clip.str_field("type") {
            Some("rect") => ClipSpec::Rect {
                x: num_field(clip, "x").unwrap_or_else(zero),
                y: num_field(clip, "y").unwrap_or_else(zero),
                w: num_field(clip, "w").unwrap_or_else(zero),
                h: num_field(clip, "h").unwrap_or_else(zero),
                r: num_field(clip, "r").unwrap_or_else(zero),
            },
            _ => ClipSpec::Circle {
                cx: num_field(clip, "cx").unwrap_or_else(zero),
                cy: num_field(clip, "cy").unwrap_or_else(zero),
                r: num_field(clip, "r").unwrap_or_else(zero),
            },
        });
    }
    Some(spec)
}

impl PropsSpec {
    /// Resolve every leaf against the engine's inputs. `None` if any present
    /// reference fails to resolve — the whole action no-ops (Clamp semantics).
    pub(crate) fn resolve(&self, engine: &StateMachineEngine) -> Option<NodeProps> {
        let num = |field: &Option<StringNumber>| -> Option<Option<f32>> {
            match field {
                None => Some(None),
                Some(v) => resolve_num(engine, v).map(Some),
            }
        };
        let mut props = NodeProps {
            x: num(&self.x)?,
            y: num(&self.y)?,
            rotate: num(&self.rotate)?,
            scale_x: num(&self.scale_x)?,
            scale_y: num(&self.scale_y)?,
            opacity: num(&self.opacity)?,
            blur: num(&self.blur)?,
            visible: self.visible,
            blend_mode: self.blend,
            ..Default::default()
        };
        if let Some(scale) = &self.scale {
            let scale = resolve_num(engine, scale)?;
            props.scale_x = Some(scale);
            props.scale_y = Some(scale);
        }
        if let Some((ax, ay)) = &self.anchor {
            props.anchor = Some((resolve_num(engine, ax)?, resolve_num(engine, ay)?));
        }
        if let Some(tint) = &self.tint {
            props.tint = Some(Tint {
                black: tint.black,
                white: tint.white,
                intensity: resolve_num(engine, &tint.intensity)?,
            });
        }
        if let Some(spot) = &self.spot {
            props.spot = Some(SpotMask {
                cx: resolve_num(engine, &spot.cx)?,
                cy: resolve_num(engine, &spot.cy)?,
                radius: resolve_num(engine, &spot.r)?,
                feather: resolve_num(engine, &spot.feather)?,
            });
        }
        if let Some(clip) = &self.clip {
            props.clip = Some(match clip {
                ClipSpec::Rect { x, y, w, h, r } => ClipRegion::Rect {
                    x: resolve_num(engine, x)?,
                    y: resolve_num(engine, y)?,
                    w: resolve_num(engine, w)?,
                    h: resolve_num(engine, h)?,
                    r: resolve_num(engine, r)?,
                },
                ClipSpec::Circle { cx, cy, r } => ClipRegion::Circle {
                    cx: resolve_num(engine, cx)?,
                    cy: resolve_num(engine, cy)?,
                    r: resolve_num(engine, r)?,
                },
            });
        }
        Some(props)
    }
}

/// `Animate` keyframes payload: dotted-key props → waypoint lists with
/// `$input`-bindable values. Uniform `scale` is lowered to X+Y at parse time.
#[derive(Debug, Clone)]
pub struct KeyframesSpec {
    entries: Vec<(Prop, Vec<StringNumber>)>,
}

fn waypoints(v: &Value) -> Option<Vec<StringNumber>> {
    if let Some(arr) = v.as_array() {
        if arr.is_empty() {
            return None;
        }
        return arr.iter().map(string_number).collect();
    }
    string_number(v).map(|n| vec![n])
}

pub(crate) fn keyframes_spec_from_json(v: &Value) -> Option<KeyframesSpec> {
    let mut entries = Vec::new();
    for (key, prop) in Prop::KEYS {
        if let Some(field) = v.get(key) {
            entries.push((*prop, waypoints(field)?));
        }
    }
    if let Some(field) = v.get("scale") {
        let values = waypoints(field)?;
        entries.push((Prop::ScaleX, values.clone()));
        entries.push((Prop::ScaleY, values));
    }
    if entries.is_empty() {
        return None;
    }
    Some(KeyframesSpec { entries })
}

impl KeyframesSpec {
    pub(crate) fn resolve(&self, engine: &StateMachineEngine) -> Option<Vec<PropKeyframes>> {
        self.entries
            .iter()
            .map(|(prop, values)| {
                let values = values
                    .iter()
                    .map(|v| resolve_num(engine, v))
                    .collect::<Option<Vec<f32>>>()?;
                Some(PropKeyframes {
                    prop: *prop,
                    values,
                    times: None,
                })
            })
            .collect()
    }
}

/// Parse `transition`/`delay` into ready `AnimateOptions`. Mirrors the wasm
/// `parse_animate_options` semantics; transition params are plain numbers
/// (no `$input` refs — feel is authored, values are bound).
pub(crate) fn animate_options_from_json(transition: Option<&Value>, delay: Option<&Value>) -> AnimateOptions {
    let transition = match transition {
        Some(v) => {
            let is_spring = v.str_field("type") == Some("spring");
            if is_spring {
                let params = if v.get("bounce").is_some() || v.get("visualDuration").is_some() {
                    SpringParams::from_bounce(
                        v.get("visualDuration").and_then(Value::as_f32).unwrap_or(0.5),
                        v.get("bounce").and_then(Value::as_f32).unwrap_or(0.2),
                    )
                } else {
                    let defaults = SpringParams::default();
                    SpringParams {
                        stiffness: v
                            .get("stiffness")
                            .and_then(Value::as_f32)
                            .unwrap_or(defaults.stiffness),
                        damping: v
                            .get("damping")
                            .and_then(Value::as_f32)
                            .unwrap_or(defaults.damping),
                        mass: v.get("mass").and_then(Value::as_f32).unwrap_or(defaults.mass),
                    }
                };
                Transition::Spring(params)
            } else {
                let easing = match v.get("ease") {
                    Some(ease) if ease.as_array().is_some() => {
                        let arr = ease.as_array().unwrap();
                        let mut e = crate::motion::EASE;
                        for (slot, item) in e.iter_mut().zip(arr) {
                            if let Some(n) = item.as_f32() {
                                *slot = n;
                            }
                        }
                        e
                    }
                    Some(ease) => ease_preset(ease.as_str().unwrap_or_default()),
                    None => crate::motion::EASE,
                };
                Transition::Tween {
                    duration: v.get("duration").and_then(Value::as_f32).unwrap_or(0.3),
                    easing,
                }
            }
        }
        None => Transition::default(),
    };
    AnimateOptions {
        transition,
        delay: delay.and_then(Value::as_f32).unwrap_or(0.0),
    }
}
