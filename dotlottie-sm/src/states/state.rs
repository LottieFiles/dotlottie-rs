use std::sync::{Arc, RwLock};

use dotlottie_player_core::{Config, DotLottiePlayer};

use crate::transition::Transition;

pub trait StateTrait {
    fn execute(&mut self, player: &mut DotLottiePlayer);
    fn reset_context(&self) -> bool;
    fn get_animation_id(&self) -> &String;
    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>>;
    fn add_transition(&mut self, transition: Arc<RwLock<Transition>>);
    fn remove_transition(&mut self, index: u32);
    fn set_reset_context(&mut self, reset_context: bool);

    // fn add_entry_action(&mut self, action: String);
    // fn add_exit_action(&mut self, action: String);
    // fn remove_entry_action(&mut self, action: String);
    // fn remove_exit_action(&mut self, action: String);
    // fn get_entry_actions(&self) -> Vec<String>;
    // fn get_exit_actions(&self) -> Vec<String>;
}

pub enum State {
    Playback {
        config: Config,
        reset_context: bool,
        animation_id: String,
        width: u32,
        height: u32,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
    Sync {
        frame_context_key: String,
        reset_context: bool,
        animation_id: String,
        width: u32,
        height: u32,
        transitions: Vec<Arc<RwLock<Transition>>>,
    },
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
    fn execute(&mut self, player: &mut DotLottiePlayer) {
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

                if animation_id != "" {
                    player.load_animation(animation_id, *width, *height);
                }

                println!("config: {:?}", config.marker);

                player.set_config(config);

                player.play();
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

    fn reset_context(&self) -> bool {
        todo!()
    }

    fn get_animation_id(&self) -> &String {
        todo!()
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

    fn add_transition(&mut self, transition: Arc<RwLock<Transition>>) {
        match self {
            State::Playback {
                config: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions,
            } => {
                transitions.push(transition);
            }
            State::Sync {
                frame_context_key: _,
                reset_context: _,
                animation_id: _,
                width: _,
                height: _,
                transitions,
            } => {
                transitions.push(transition);
            }
        }
    }

    fn remove_transition(&mut self, index: u32) {
        todo!()
    }

    fn set_reset_context(&mut self, reset_context: bool) {
        todo!()
    }
}
