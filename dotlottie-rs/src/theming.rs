use serde_json::{json, Value};
use std::collections::HashMap;

pub fn transform_theme_to_lottie_slots(
    theme_json: &str,
    active_animation_id: &str,
) -> Result<String, serde_json::Error> {
    let theme: Value = serde_json::from_str(theme_json)?;
    let rules = theme["rules"]
        .as_array()
        .ok_or_else(|| serde::de::Error::custom("Invalid rules field"))?;

    let mut lottie_slots = HashMap::new();

    for rule in rules {
        if !should_process_rule(rule, active_animation_id) {
            continue;
        }

        let slot_id = rule["id"].as_str().unwrap_or("");
        let slot_type = rule["type"].as_str().unwrap_or("");

        let p = match slot_type {
            "Image" => handle_image_slot(rule),
            "Gradient" => handle_gradient_slot(rule),
            "Scalar" => handle_scalar_slot(rule),
            _ => handle_other_slot_types(rule),
        };

        lottie_slots.insert(slot_id.to_string(), json!({ "p": p }));
    }

    let lottie_slots_json = serde_json::to_string(&lottie_slots)?;
    Ok(lottie_slots_json)
}

fn should_process_rule(rule: &Value, active_animation_id: &str) -> bool {
    if rule.get("animations").is_none() {
        return true;
    }

    let empty_vec = vec![];
    let animations = rule["animations"].as_array().unwrap_or(&empty_vec);
    animations
        .iter()
        .any(|anim| anim.as_str() == Some(active_animation_id))
}

fn handle_image_slot(rule: &Value) -> Value {
    if let Some(value) = rule["value"].as_object() {
        let mut image_data = json!({});

        if let Some(width) = value.get("width").and_then(|v| v.as_u64()) {
            image_data["w"] = json!(width);
        }

        if let Some(height) = value.get("height").and_then(|v| v.as_u64()) {
            image_data["h"] = json!(height);
        }

        if let Some(path) = value.get("path").and_then(|v| v.as_str()) {
            image_data["u"] = json!(path); // should be the path
            image_data["p"] = json!(path.split('/').last().unwrap_or("")); // should be the file name
            image_data["e"] = json!(0);
        }

        if let Some(data_url) = value.get("dataUrl").and_then(|v| v.as_str()) {
            image_data["p"] = json!(data_url);
            image_data["e"] = json!(1);
        }

        image_data
    } else {
        json!({})
    }
}

fn handle_gradient_slot(rule: &Value) -> Value {
    if let Some(keyframes) = rule["keyframes"].as_array() {
        let lottie_keyframes: Vec<Value> = keyframes
            .iter()
            .map(|keyframe| {
                let mut frame_data = json!({});

                if let Some(frame_number) = keyframe["frame"].as_u64() {
                    frame_data["t"] = json!(frame_number);
                }

                if let Some(value) = keyframe["value"].as_array() {
                    let mut gradient_data = vec![];
                    let mut transparency_data = vec![];
                    let mut alpha_present = false;

                    for stop in value {
                        if let Some(color) = stop["color"].as_array() {
                            if color.len() == 4 {
                                alpha_present = true;
                                break;
                            }
                        }
                    }

                    for stop in value {
                        if let Some(offset) = stop["offset"].as_f64() {
                            if let Some(color) = stop["color"].as_array() {
                                gradient_data.push(offset);
                                for component in color.iter().take(3) {
                                    gradient_data.push(component.as_f64().unwrap_or(0.0));
                                }

                                let alpha = if color.len() == 4 {
                                    color[3].as_f64().unwrap_or(1.0)
                                } else if alpha_present {
                                    1.0
                                } else {
                                    continue;
                                };

                                transparency_data.push(offset);
                                transparency_data.push(alpha);
                            }
                        }
                    }

                    gradient_data.extend(transparency_data);

                    frame_data["s"] = json!(gradient_data);
                }

                if let Some(in_tangent) = keyframe["inTangent"].as_object() {
                    if let (Some(x), Some(y)) = (in_tangent.get("x"), in_tangent.get("y")) {
                        frame_data["i"] = json!({ "x": x, "y": y });
                    }
                }

                if let Some(out_tangent) = keyframe["outTangent"].as_object() {
                    if let (Some(x), Some(y)) = (out_tangent.get("x"), out_tangent.get("y")) {
                        frame_data["o"] = json!({ "x": x, "y": y });
                    }
                }

                if let Some(hold) = keyframe["hold"].as_bool() {
                    frame_data["h"] = json!(if hold { 1 } else { 0 });
                }

                frame_data
            })
            .collect();

        json!({
            "k": json!({
                "a": 1,
                "k": lottie_keyframes
            }),
            "p": keyframes[0]["value"].as_array().map(|v| v.len()).unwrap_or(0)
        })
    } else if let Some(value) = rule["value"].as_array() {
        let mut gradient_data = vec![];
        let mut transparency_data = vec![];
        let mut alpha_present = false;

        for stop in value {
            if let Some(color) = stop["color"].as_array() {
                if color.len() == 4 {
                    alpha_present = true;
                    break;
                }
            }
        }

        for stop in value {
            if let Some(offset) = stop["offset"].as_f64() {
                if let Some(color) = stop["color"].as_array() {
                    gradient_data.push(offset);
                    for component in color.iter().take(3) {
                        gradient_data.push(component.as_f64().unwrap_or(0.0));
                    }

                    let alpha = if color.len() == 4 {
                        color[3].as_f64().unwrap_or(1.0)
                    } else if alpha_present {
                        1.0
                    } else {
                        continue;
                    };

                    transparency_data.push(offset);
                    transparency_data.push(alpha);
                }
            }
        }

        gradient_data.extend(transparency_data);

        json!({
            "k": json!({
                "a": 0,
                "k": gradient_data
            }),
            "p": value.len()
        })
    } else {
        json!({})
    }
}

fn handle_scalar_slot(rule: &Value) -> Value {
    if let Some(keyframes) = rule["keyframes"].as_array() {
        let lottie_keyframes: Vec<Value> = keyframes.iter().map(handle_scalar_keyframe).collect();

        json!({
            "a": 1,
            "k": json!(lottie_keyframes)
        })
    } else if let Some(value) = rule["value"].as_f64() {
        json!({
            "a": 0,
            "k": json!(vec![value])
        })
    } else {
        json!({})
    }
}

fn handle_scalar_keyframe(keyframe: &Value) -> Value {
    let mut frame_data = json!({});

    if let Some(frame) = keyframe["frame"].as_u64() {
        frame_data["t"] = json!(frame);
    }

    if let Some(value) = keyframe["value"].as_f64() {
        frame_data["s"] = json!(vec![value]);
    }

    if let Some(in_tangent) = keyframe["inTangent"].as_object() {
        if let (Some(x), Some(y)) = (in_tangent.get("x"), in_tangent.get("y")) {
            frame_data["i"] = json!({ "x": x, "y": y });
        }
    }

    if let Some(out_tangent) = keyframe["outTangent"].as_object() {
        if let (Some(x), Some(y)) = (out_tangent.get("x"), out_tangent.get("y")) {
            frame_data["o"] = json!({ "x": x, "y": y });
        }
    }

    if let Some(value_in_tangent) = keyframe["valueInTangent"].as_array() {
        frame_data["ti"] = json!(value_in_tangent);
    }

    if let Some(value_out_tangent) = keyframe["valueOutTangent"].as_array() {
        frame_data["to"] = json!(value_out_tangent);
    }

    if let Some(hold) = keyframe["hold"].as_bool() {
        frame_data["h"] = json!(if hold { 1 } else { 0 });
    }

    frame_data
}

fn handle_other_slot_types(rule: &Value) -> Value {
    if let Some(keyframes) = rule["keyframes"].as_array() {
        let lottie_keyframes: Vec<Value> = keyframes.iter().map(handle_generic_keyframe).collect();

        json!({
            "a": if keyframes.len() > 1 { 1 } else { 0 },
            "k": if keyframes.len() > 1 { json!(lottie_keyframes) } else { lottie_keyframes[0].clone() }
        })
    } else if let Some(value) = rule["value"].as_array() {
        json!({
            "a": 0,
            "k": json!(value)
        })
    } else {
        json!({})
    }
}

fn handle_generic_keyframe(keyframe: &Value) -> Value {
    let mut frame_data = json!({});

    if let Some(frame) = keyframe["frame"].as_u64() {
        frame_data["t"] = json!(frame);
    }

    if let Some(value) = keyframe["value"].as_array() {
        frame_data["s"] = json!(value);
    }

    if let Some(in_tangent) = keyframe["inTangent"].as_object() {
        if let (Some(x), Some(y)) = (in_tangent.get("x"), in_tangent.get("y")) {
            frame_data["i"] = json!({ "x": x, "y": y });
        }
    }

    if let Some(out_tangent) = keyframe["outTangent"].as_object() {
        if let (Some(x), Some(y)) = (out_tangent.get("x"), out_tangent.get("y")) {
            frame_data["o"] = json!({ "x": x, "y": y });
        }
    }

    if let Some(value_in_tangent) = keyframe["valueInTangent"].as_array() {
        frame_data["ti"] = json!(value_in_tangent);
    }

    if let Some(value_out_tangent) = keyframe["valueOutTangent"].as_array() {
        frame_data["to"] = json!(value_out_tangent);
    }

    if let Some(hold) = keyframe["hold"].as_bool() {
        frame_data["h"] = json!(if hold { 1 } else { 0 });
    }

    frame_data
}
