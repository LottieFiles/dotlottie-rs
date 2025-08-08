#[derive(Debug, Clone, PartialEq, Default)]
#[repr(C)]
pub struct OpenUrlPolicy {
    pub whitelist: Vec<String>,
    pub require_user_interaction: bool,
}

impl OpenUrlPolicy {
    pub fn new(whitelist: Vec<String>, require_user_interaction: bool) -> Self {
        Self {
            whitelist,
            require_user_interaction,
        }
    }
}
