use serde::Deserialize;
use serde_json_core::de::from_str;
use std::string::String;
use std::vec::Vec;

#[derive(Clone, Deserialize)]
pub struct Marker {
    pub name: String,
    pub time: f32,
    pub duration: f32,
}

#[derive(Deserialize)]
struct Lottie {
    markers: Vec<Marker>,
}

#[inline]
pub fn extract_markers(json_data: &str) -> Vec<Marker> {
    let container: Result<(Lottie, _), _> = from_str(json_data);

    match container {
        Ok((container, _)) => container.markers,
        Err(_) => vec![],
    }
}
