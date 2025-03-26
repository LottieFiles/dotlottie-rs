#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(C)]
pub enum OpenUrlMode {
    Deny,
    Interaction,
    Allow,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct OpenUrl {
    pub mode: OpenUrlMode,
    pub whitelist: Vec<String>,
}

impl Default for OpenUrl {
    fn default() -> Self {
        Self {
            mode: OpenUrlMode::Interaction,
            whitelist: vec![],
        }
    }
}

impl OpenUrl {
    pub fn new(mode: OpenUrlMode, whitelist: Vec<String>) -> Self {
        Self { mode, whitelist }
    }
}
