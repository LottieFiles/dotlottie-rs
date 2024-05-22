use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{Config, DotLottiePlayerContainer};

use super::transitions::Transition;

pub trait StateTrait {
    fn execute(&mut self, player: &Rc<RwLock<DotLottiePlayerContainer>>);
    fn get_reset_context_key(&self) -> &String;
    fn get_animation_id(&self) -> &String;
    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>>;
    fn add_transition(&mut self, transition: Transition);
    fn remove_transition(&mut self, index: u32);
    fn get_config(&self) -> Option<&Config>;
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
        config: Config,
        reset_context: String,
        animation_id: String,
        width: u32,
        height: u32,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
    Sync {
        frame_context_key: String,
        reset_context: String,
        animation_id: String,
        width: u32,
        height: u32,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Playback {
                config: _,
                reset_context,
                animation_id,
                width,
                height,
                transitions,
            } => f
                .debug_struct("State::Playback")
                // .field("config", &(config))
                .field("reset_context", reset_context)
                .field("animation_id", animation_id)
                .field("width", width)
                .field("height", height)
                .field("transitions", transitions)
                .finish(),

            State::Sync {
                frame_context_key,
                reset_context,
                animation_id,
                width,
                height,
                transitions,
            } => f
                .debug_struct("State::Sync")
                .field("frame_context_key", frame_context_key)
                .field("reset_context", reset_context)
                .field("animation_id", animation_id)
                .field("width", width)
                .field("height", height)
                .field("transitions", transitions)
                .finish(),
        }
    }
}

impl State {
    pub fn as_str(&self) -> &str {
        match self {
            State::Playback {
                config: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => "Playback",
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => "Sync",
        }
    }
}

impl StateTrait for State {
    fn execute(&mut self, player: &Rc<RwLock<DotLottiePlayerContainer>>) {
        match self {
            State::Playback {
                config,
                reset_context: _,
                animation_id,
                width,
                height,
                transitions: _,
            } => {
                let config = config.clone();

                // Tell player to load new animation
                if !animation_id.is_empty() {
                    player
                        .write()
                        .unwrap()
                        .load_animation(&animation_id, *width, *height);
                }

                println!("config: {:?}", config);

                // Set the config
                player.write().unwrap().set_config(config);
                player.write().unwrap().play();
            }
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => {
                todo!()
            }
        }
    }

    fn get_reset_context_key(&self) -> &String {
        match self {
            State::Playback {
                config: _,
                reset_context,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => reset_context,
            State::Sync {
                frame_context_key: _,
                reset_context,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => reset_context,
        }
    }

    fn get_animation_id(&self) -> &String {
        match self {
            State::Playback {
                config: _,
                reset_context: _,
                animation_id,
                width: _,
                height: _,
                transitions: _,
            } => animation_id,
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id,
                width: _,
                height: _,
                transitions: _,
            } => animation_id,
        }
    }

    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>> {
        match self {
            State::Playback {
                config: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions,
            } => transitions,
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions,
            } => transitions,
        }
    }

    fn add_transition(&mut self, transition: Transition) {
        match self {
            State::Playback {
                config: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions,
            } => {
                transitions.push(Arc::new(RwLock::new(transition)));
            }
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions,
            } => {
                transitions.push(Arc::new(RwLock::new(transition)));
            }
        }
    }

    fn remove_transition(&mut self, _index: u32) {
        todo!()
    }

    fn get_config(&self) -> Option<&Config> {
        match self {
            State::Playback {
                config,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => return Some(config),
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions: _,
            } => return None,
        }
    }

    // fn set_reset_context(&mut self, reset_context: bool) {
    //     todo!()
    // }
}
