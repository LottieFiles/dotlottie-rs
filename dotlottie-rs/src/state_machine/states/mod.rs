use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::dotlottie_player::Config;
use crate::uniffi::DotLottiePlayerContainer;

use super::transitions::Transition;

pub trait StateTrait {
    fn execute(
        &self,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
        string_context: &HashMap<String, String>,
        bool_context: &HashMap<String, bool>,
        numeric_context: &HashMap<String, f32>,
    ) -> i32;
    fn get_reset_context_key(&self) -> &String;
    fn get_animation_id(&self) -> Option<&String>;
    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>>;
    fn add_transition(&mut self, transition: Transition);
    fn get_config(&self) -> Option<&Config>;
    fn get_name(&self) -> String;
    fn get_type(&self) -> String;
    // fn set_reset_context(&mut self, reset_context: bool);

    // fn add_entry_action(&mut self, action: String);
    // fn add_exit_action(&mut self, action: String);
    // fn remove_entry_action(&mut self, action: String);
    // fn remove_exit_action(&mut self, action: String);
    // fn get_entry_actions(&self) -> Vec<String>;
    // fn get_exit_actions(&self) -> Vec<String>;
}

#[derive(Clone, Debug)]
pub enum State {
    Playback {
        name: String,
        config: Config,
        reset_context: String,
        animation_id: String,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
    Sync {
        name: String,
        config: Config,
        frame_context_key: String,
        reset_context: String,
        animation_id: String,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
    Global {
        name: String,
        reset_context: String,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
}

impl State {
    pub fn as_str(&self) -> &str {
        match self {
            State::Playback { .. } => "Playback",
            State::Sync { .. } => "Sync",
            State::Global { .. } => "Global",
        }
    }
}

impl StateTrait for State {
    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    fn execute(
        &self,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
        _: &HashMap<String, String>,
        _: &HashMap<String, bool>,
        numeric_context: &HashMap<String, f32>,
    ) -> i32 {
        match self {
            State::Playback {
                config,
                animation_id,
                ..
            } => {
                let config = config.clone();
                let autoplay = config.autoplay;

                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();

                    // Tell player to load new animation
                    if !animation_id.is_empty() {
                        player_read.load_animation(animation_id, size.0, size.1);
                    }

                    player_read.set_config(config);

                    if autoplay {
                        player_read.play();

                        return 2;
                    } else {
                        player_read.pause();

                        return 3;
                    }
                } else {
                    return 1;
                }
            }
            State::Sync {
                config,
                frame_context_key,
                animation_id,
                ..
            } => {
                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();
                    let frame = numeric_context.get(frame_context_key);

                    // Tell player to load new animation
                    if !animation_id.is_empty() {
                        player_read.load_animation(animation_id, size.0, size.1);
                    }

                    player_read.set_config(config.clone());

                    if let Some(frame_value) = frame {
                        let ret = player_read.set_frame(*frame_value);

                        if ret {
                            return 4;
                        }
                    }
                }
            }
            State::Global { .. } => {}
        }

        0
    }

    fn get_reset_context_key(&self) -> &String {
        match self {
            State::Playback { reset_context, .. } => reset_context,
            State::Sync { reset_context, .. } => reset_context,
            State::Global { reset_context, .. } => reset_context,
        }
    }

    fn get_animation_id(&self) -> Option<&String> {
        match self {
            State::Playback { animation_id, .. } => Some(animation_id),
            State::Sync { animation_id, .. } => Some(animation_id),
            State::Global { .. } => None,
        }
    }

    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>> {
        match self {
            State::Playback { transitions, .. } => transitions,
            State::Sync { transitions, .. } => transitions,
            State::Global { transitions, .. } => transitions,
        }
    }

    fn add_transition(&mut self, transition: Transition) {
        match self {
            State::Playback { transitions, .. } => {
                transitions.push(Arc::new(RwLock::new(transition)));
            }
            State::Sync { transitions, .. } => {
                transitions.push(Arc::new(RwLock::new(transition)));
            }
            State::Global { transitions, .. } => {
                transitions.push(Arc::new(RwLock::new(transition)));
            }
        }
    }

    fn get_config(&self) -> Option<&Config> {
        match self {
            State::Playback { config, .. } => Some(config),
            State::Sync { .. } => None,
            State::Global { .. } => None,
        }
    }

    fn get_name(&self) -> String {
        match self {
            State::Playback { name, .. } => name.to_string(),
            State::Sync { name, .. } => name.to_string(),
            State::Global { name, .. } => name.to_string(),
        }
    }

    fn get_type(&self) -> String {
        match self {
            State::Playback { .. } => "PlaybackState".to_string(),
            State::Sync { .. } => "SyncState".to_string(),
            State::Global { .. } => "GlobalState".to_string(),
        }
    }

    // fn set_reset_context(&mut self, reset_context: bool) {
    //     todo!()
    // }
}
