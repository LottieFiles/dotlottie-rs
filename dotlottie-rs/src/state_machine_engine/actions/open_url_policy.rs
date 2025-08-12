#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct OpenUrlPolicy {
    pub whitelist: Vec<String>,
    pub require_user_interaction: bool,
}

impl Default for OpenUrlPolicy {
    fn default() -> Self {
        return OpenUrlPolicy {
            whitelist: vec![],
            require_user_interaction: true,
        };
    }
}

impl OpenUrlPolicy {
    pub fn new(whitelist: Vec<String>, require_user_interaction: bool) -> Self {
        Self {
            whitelist,
            require_user_interaction,
        }
    }
}
