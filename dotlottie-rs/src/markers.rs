use std::ffi::CString;

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

/// Extracts markers from Lottie JSON data.
///
/// Returns parallel arrays: (names, data) where data[i] = (time, duration) for names[i].
pub fn extract_markers(json_data: &str) -> (Vec<CString>, Vec<(f32, f32)>) {
    let mut names = Vec::new();
    let mut data = Vec::new();

    let Ok(lottie) = serde_json::from_str::<Lottie>(json_data) else {
        return (names, data);
    };

    for marker in lottie.markers {
        let name = marker.name.trim();

        if name.is_empty() || marker.duration < 0.0 || marker.time < 0.0 {
            continue;
        }

        // Skip duplicates (keep first occurrence)
        if let Ok(c_name) = CString::new(name) {
            if !names
                .iter()
                .any(|n: &CString| n.as_c_str() == c_name.as_c_str())
            {
                names.push(c_name);
                data.push((marker.time, marker.duration));
            }
        }
    }

    (names, data)
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

        let (names, data) = extract_markers(&json_data);

        assert_eq!(names.len(), 2);
        assert_eq!(data.len(), 2);
        assert_eq!(names[0].to_str().unwrap(), "Marker1");
        assert_eq!(data[0], (0.5, 1.5));
        assert_eq!(names[1].to_str().unwrap(), "Marker2");
        assert_eq!(data[1], (1.5, 2.5));
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

        let (names, data) = extract_markers(&json_data);

        assert_eq!(names.len(), 1);
        assert_eq!(data.len(), 1);
        assert_eq!(names[0].to_str().unwrap(), "Marker2");
    }

    #[test]
    fn test_extract_markers_invalid_json() {
        let json_data = "This is not a valid JSON".to_string();
        let (names, data) = extract_markers(&json_data);
        assert!(names.is_empty());
        assert!(data.is_empty());
    }

    #[test]
    fn test_extract_markers_wrong_structure() {
        let json_data = json!({"unexpected_field": "unexpected_value"}).to_string();
        let (names, data) = extract_markers(&json_data);
        assert!(names.is_empty());
        assert!(data.is_empty());
    }

    #[test]
    fn test_extract_markers_empty() {
        let json_data = json!({}).to_string();
        let (names, data) = extract_markers(&json_data);
        assert!(names.is_empty());
        assert!(data.is_empty());
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

        let (names, data) = extract_markers(&json_data);

        // Keeps first occurrence
        assert_eq!(names.len(), 1);
        assert_eq!(data.len(), 1);
        assert_eq!(names[0].to_str().unwrap(), "Marker1");
        assert_eq!(data[0], (0.5, 1.5));
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

        let (names, data) = extract_markers(&json_data);

        assert_eq!(names.len(), 1);
        assert_eq!(names[0].to_str().unwrap(), "Marker2");
        assert_eq!(data[0], (1.5, 2.5));
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

        let (names, data) = extract_markers(&json_data);

        assert_eq!(names.len(), 1);
        assert_eq!(names[0].to_str().unwrap(), "Marker2");
        assert_eq!(data[0], (1.5, 2.5));
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

        let (names, data) = extract_markers(&json_data);

        assert_eq!(names.len(), 2);
        assert_eq!(names[0].to_str().unwrap(), "Marker1");
        assert_eq!(data[0], (1e10, 1.5));
        assert_eq!(names[1].to_str().unwrap(), "Marker2");
        assert_eq!(data[1], (1.5, 2.5));
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

        let (names, data) = extract_markers(&json_data);

        assert_eq!(names.len(), 2);
        assert_eq!(names[0].to_str().unwrap(), "Marker1");
        assert_eq!(data[0], (0.5, 1.5));
        assert_eq!(names[1].to_str().unwrap(), "Marker2");
        assert_eq!(data[1], (1.5, 2.5));
    }
}
