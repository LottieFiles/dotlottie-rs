use crate::{DotLottiePlayerContainer, StateMachineEngine};
use std::{rc::Rc, sync::RwLock};

pub struct NativeOpenUrl;

impl NativeOpenUrl {
    pub fn open_url(
        url: &str,
        target: &str,
        _engine: &StateMachineEngine,
        _player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<(), String> {
        #[cfg(target_os = "emscripten")]
        {
            let command = if target.is_empty() {
                format!("OpenUrl: {}", url)
            } else {
                format!("OpenUrl: {} | Target: {}", url, target)
            };

            _player
                .read()
                .unwrap()
                .emit_state_machine_observer_on_custom_event(command);

            return Ok(());
        }

        #[cfg(not(target_os = "emscripten"))]
        {
            let _ = target.to_lowercase();
            let command = format!("OpenUrl: {}", url);
            _engine.observe_framework_open_url_event(&command);
            Ok(())
        }
    }
}
