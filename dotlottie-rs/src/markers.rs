use std::collections::HashMap;

pub struct Marker {
    pub name: String,
    pub time: f32,
    pub duration: f32,
}

pub type MarkersMap = HashMap<String, (f32, f32)>;

pub fn extract_markers(json_data: &str) -> MarkersMap {
    jzon::parse(json_data)
        .ok()
        .and_then(|parsed| parsed["markers"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|marker| {
            let name = marker["cm"].as_str()?.trim().to_string();
            let duration = marker["dr"].as_f32()?;
            let time = marker["tm"].as_f32()?;
            if name.is_empty() || duration < 0.0 || time < 0.0 {
                return None;
            }
            Some((name, (time, duration)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markers_normal() {
        let json_data = r#"{"markers":[{"cm":"Marker1","dr":1.5,"tm":0.5},{"cm":"Marker2","dr":2.5,"tm":1.5}]}"#;
        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 2);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (0.5, 1.5));
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_empty_name() {
        let json_data =
            r#"{"markers":[{"cm":"","dr":1.5,"tm":0.5},{"cm":"Marker2","dr":2.5,"tm":1.5}]}"#;
        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_invalid_json() {
        let json_data = "This is not a valid JSON";
        assert!(extract_markers(&json_data).is_empty());
    }

    #[test]
    fn test_extract_markers_wrong_structure() {
        let json_data = r#"{"unexpected_field":"unexpected_value"}"#;
        let markers = extract_markers(&json_data);
        assert_eq!(markers.len(), 0);
    }

    #[test]
    fn test_extract_markers_empty() {
        let json_data = r#"{}"#;
        let markers = extract_markers(&json_data);
        assert_eq!(markers.len(), 0);
    }

    #[test]
    fn test_extract_markers_duplicate_names() {
        let json_data = r#"{"markers":[{"cm":"Marker1","dr":1.5,"tm":0.5},{"cm":"Marker1","dr":2.5,"tm":1.5}]}"#;

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_negative_duration() {
        let json_data = r#"{"markers":[{"cm":"Marker1","dr":-1.5,"tm":0.5},{"cm":"Marker2","dr":2.5,"tm":1.5}]}"#;

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_negative_time() {
        let json_data = r#"{"markers":[{"cm":"Marker1","dr":1.5,"tm":-0.5},{"cm":"Marker2","dr":2.5,"tm":1.5}]}"#;

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 1);
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_extract_markers_large_numbers() {
        let json_data = r#"{"markers":[{"cm":"Marker1","dr":1.5,"tm":1e10},{"cm":"Marker2","dr":2.5,"tm":1.5}]}"#;

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 2);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (1e10, 1.5));
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }

    #[test]
    fn test_trim_marker_name() {
        let json_data = r#"{"markers":[{"cm":" Marker1 ","dr":1.5,"tm":0.5},{"cm":"Marker2","dr":2.5,"tm":1.5}]}"#;

        let markers = extract_markers(&json_data);

        assert_eq!(markers.len(), 2);
        assert!(markers.contains_key("Marker1"));
        assert_eq!(markers["Marker1"], (0.5, 1.5));
        assert!(markers.contains_key("Marker2"));
        assert_eq!(markers["Marker2"], (1.5, 2.5));
    }
}
