use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Marker {
    #[serde(rename = "cm")]
    pub name: String,
    #[serde(rename = "dr")]
    pub duration: f32,
    #[serde(rename = "tm")]
    pub time: f32,
}

#[derive(Serialize, Deserialize)]
struct Lottie {
    markers: Vec<Marker>,
}

pub type MarkersMap = HashMap<String, (f32, f32)>;

pub fn extract_markers(json_data: &str) -> MarkersMap {
    let mut markers_map = HashMap::new();

    match serde_json::from_str::<Lottie>(json_data) {
        Ok(lottie) => {
            for marker in lottie.markers {
                let name = marker.name.trim();

                if name.is_empty() || marker.duration < 0.0 || marker.time < 0.0 {
                    continue;
                }

                markers_map.insert(name.to_string(), (marker.time, marker.duration));
            }

            markers_map
        }
        Err(_) => markers_map,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_markers_normal() {
        let json_data = json!({
            "markers": [
                {"cm": "Marker1", "dr": 1.5, "tm": 0.5},
                {"cm": "Marker2", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 2);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (0.5, 1.5));
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_empty_name() {
        let json_data = json!({
            "markers": [
                {"cm": "", "dr": 1.5, "tm": 0.5},
                {"cm": "Marker2", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker2"));
    }

    #[test]
    fn test_extract_markers_invalid_json() {
        let json_data = "This is not a valid JSON".to_string();
        assert!(extract_markers(&json_data).is_empty());
    }

    #[test]
    fn test_extract_markers_wrong_structure() {
        let json_data = json!({"unexpected_field": "unexpected_value"}).to_string();
        assert!(extract_markers(&json_data).is_empty());
    }

    #[test]
    fn test_extract_markers_empty() {
        let json_data = json!({}).to_string();
        assert!(extract_markers(&json_data).is_empty());
    }

    #[test]
    fn test_extract_markers_duplicate_names() {
        let json_data = json!({
            "markers": [
                {"cm": "Marker1", "dr": 1.5, "tm": 0.5},
                {"cm": "Marker1", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_negative_duration() {
        let json_data = json!({
            "markers": [
                {"cm": "Marker1", "dr": -1.5, "tm": 0.5},
                {"cm": "Marker2", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_negative_time() {
        let json_data = json!({
            "markers": [
                {"cm": "Marker1", "dr": 1.5, "tm": -0.5},
                {"cm": "Marker2", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_large_numbers() {
        let json_data = json!({
            "markers": [
                {"cm": "Marker1", "dr": 1.5, "tm": 1e10},
                {"cm": "Marker2", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 2);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (1e10, 1.5));
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_trim_marker_name() {
        let json_data = json!({
            "markers": [
                {"cm": " Marker1 ", "dr": 1.5, "tm": 0.5},
                {"cm": "Marker2", "dr": 2.5, "tm": 1.5}
            ]
        })
        .to_string();

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 2);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (0.5, 1.5));
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }
}
