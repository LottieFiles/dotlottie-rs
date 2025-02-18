#[derive(Debug, Clone, PartialEq, Copy)]
pub enum OpenURLMode {
    Deny,
    Interaction,
    Allow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenURL {
    pub mode: OpenURLMode,
    pub whitelist: Vec<String>,
}

impl Default for OpenURL {
    fn default() -> Self {
        Self {
            mode: OpenURLMode::Interaction,
            whitelist: vec![],
        }
    }
}

impl OpenURL {
    pub fn new(mode: OpenURLMode, whitelist: Vec<String>) -> Self {
        Self { mode, whitelist }
    }
}
