use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestInitial {
    pub animation: Option<String>,
    pub state_machine: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestTheme {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestStateMachine {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestAnimation {
    pub id: String,
    pub name: Option<String>,
    pub themes: Option<Vec<String>>,
    pub background: Option<String>,
    pub initial_theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: Option<String>,
    pub generator: Option<String>,

    pub initial: Option<ManifestInitial>,

    pub animations: Vec<ManifestAnimation>,
    pub themes: Option<Vec<ManifestTheme>>,
    pub state_machines: Option<Vec<ManifestStateMachine>>,
}
