#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Mode {
    Deny,
    Interaction,
    Whitelist,
    Allow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenURL {
    pub mode: Mode,
    pub whitelist: Vec<String>,
}

impl Default for OpenURL {
    fn default() -> Self {
        Self {
            mode: Mode::Interaction,
            whitelist: vec![],
        }
    }
}

impl OpenURL {
    pub fn new(mode: Mode, whitelist: Vec<String>) -> Self {
        Self {
            mode,
            whitelist,
        }
    }
}
