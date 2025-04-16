use crate::StateMachineEngine;

pub struct NativeOpenUrl;

impl NativeOpenUrl {
    pub fn open_url(url: &str, target: &str, engine: &StateMachineEngine) -> Result<(), String> {
        let _ = target.to_lowercase();
        let command = if target.is_empty() {
            format!("OpenUrl: {}", url)
        } else {
            format!("OpenUrl: {} | Target: {}", url, target)
        };

        engine.observe_framework_open_url_event(&command);
        Ok(())
    }
}
