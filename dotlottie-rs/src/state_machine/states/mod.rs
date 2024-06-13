use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{Config, DotLottiePlayerContainer};

use super::transitions::Transition;

pub trait StateTrait {
    fn execute(&self, player: &Rc<RwLock<DotLottiePlayerContainer>>);
    fn get_reset_context_key(&self) -> &String;
    fn get_animation_id(&self) -> &String;
    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>>;
    fn add_transition(&mut self, transition: Transition);
    fn remove_transition(&mut self, index: u32);
    fn get_config(&self) -> Option<&Config>;
    fn get_name(&self) -> String;
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
        frame_context_key: String,
        reset_context: String,
        animation_id: String,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Playback {
                name: _,
                config: _,
                reset_context,
                animation_id,
                transitions,
            } => f
                .debug_struct("State::Playback")
                // .field("config", &(config))
                .field("reset_context", reset_context)
                .field("animation_id", animation_id)
                .field("transitions", transitions)
                .finish(),

            State::Sync {
                name: _,
                frame_context_key,
                reset_context,
                animation_id,
                transitions,
            } => f
                .debug_struct("State::Sync")
                .field("frame_context_key", frame_context_key)
                .field("reset_context", reset_context)
                .field("animation_id", animation_id)
                .field("transitions", transitions)
                .finish(),
        }
    }
}

impl State {
    pub fn as_str(&self) -> &str {
        match self {
            State::Playback {
                name: _,
                config: _,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => "Playback",
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => "Sync",
        }
    }
}

impl StateTrait for State {
    fn execute(&self, player: &Rc<RwLock<DotLottiePlayerContainer>>) {
        match self {
            State::Playback {
                name: _,
                config,
                reset_context: _,
                animation_id,
                transitions: _,
            } => {
                let config = config.clone();
                let autoplay = config.autoplay;
                let size = player.read().unwrap().size();

                // Tell player to load new animation
                if !animation_id.is_empty() {
                    player
                        .read()
                        .unwrap()
                        .load_animation(animation_id, size.0, size.1);
                }

                // We have to use read otherwise it will deadlock
                player.read().unwrap().set_config(config);

                if autoplay {
                    player.read().unwrap().play();
                }
            }
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => {
                todo!()
            }
        }
    }

    fn get_reset_context_key(&self) -> &String {
        match self {
            State::Playback {
                name: _,
                config: _,
                reset_context,
                animation_id: _,
                transitions: _,
            } => reset_context,
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context,
                animation_id: _,
                transitions: _,
            } => reset_context,
        }
    }

    fn get_animation_id(&self) -> &String {
        match self {
            State::Playback {
                name: _,
                config: _,
                reset_context: _,
                animation_id,
                transitions: _,
            } => animation_id,
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context: _,
                animation_id,
                transitions: _,
            } => animation_id,
        }
    }

    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>> {
        match self {
            State::Playback {
                name: _,
                config: _,
                reset_context: _,
                animation_id: _,
                transitions,
            } => transitions,
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                transitions,
            } => transitions,
        }
    }

    fn add_transition(&mut self, transition: Transition) {
        match self {
            State::Playback {
                name: _,
                config: _,
                reset_context: _,
                animation_id: _,
                transitions,
            } => {
                transitions.push(Arc::new(RwLock::new(transition)));
            }
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
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
                name: _,
                config,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => Some(config),
            State::Sync {
                name: _,
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => None,
        }
    }

    fn get_name(&self) -> String {
        match self {
            State::Playback {
                name,
                config: _,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => name.to_string(),
            State::Sync {
                name,
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                transitions: _,
            } => name.to_string(),
        }
    }

    // fn set_reset_context(&mut self, reset_context: bool) {
    //     todo!()
    // }
}
