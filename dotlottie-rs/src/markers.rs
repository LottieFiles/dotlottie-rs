use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Result;

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

pub fn extract_markers(json_data: &str) -> Result<MarkersMap> {
    let lottie: Lottie = serde_json::from_str(json_data)?;

    let mut markers_map = HashMap::new();

    for marker in lottie.markers {
        let name = marker.name.trim();

        if name.is_empty() {
            continue;
        }

        markers_map.insert(name.to_string(), (marker.time, marker.duration));
    }

    Ok(markers_map)
}
