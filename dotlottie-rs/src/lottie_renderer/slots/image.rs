use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSlot {
    #[serde(skip_serializing_if = "Option::is_none", rename = "w")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "h")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "u")]
    pub directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "p")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "e")]
    pub embed: Option<u8>,
}

impl ImageSlot {
    pub fn from_path(path: String) -> Self {
        let filename = path.split('/').next_back().unwrap_or(&path).to_string();
        let dir = path.rsplit_once('/').map(|x| x.0.to_string());

        Self {
            width: None,
            height: None,
            directory: dir,
            path: Some(filename),
            embed: Some(0),
        }
    }

    pub fn from_data_url(data_url: String) -> Self {
        Self {
            width: None,
            height: None,
            directory: None,
            path: Some(data_url),
            embed: Some(1),
        }
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}
