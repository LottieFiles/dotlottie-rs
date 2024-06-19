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

impl State {
    pub fn as_str(&self) -> &str {
        match self {
            State::Playback { .. } => "Playback",
            State::Sync { .. } => "Sync",
        }
    }
}

impl StateTrait for State {
    fn execute(&self, player: &Rc<RwLock<DotLottiePlayerContainer>>) {
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
                    }
                }
            }
            State::Sync { .. } => {}
        }
    }

    fn get_reset_context_key(&self) -> &String {
        match self {
            State::Playback { reset_context, .. } => reset_context,
            State::Sync { reset_context, .. } => reset_context,
        }
    }

    fn get_animation_id(&self) -> &String {
        match self {
            State::Playback { animation_id, .. } => animation_id,
            State::Sync { animation_id, .. } => animation_id,
        }
    }

    fn get_transitions(&self) -> &Vec<Arc<RwLock<Transition>>> {
        match self {
            State::Playback { transitions, .. } => transitions,
            State::Sync { transitions, .. } => transitions,
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
        }
    }

    fn get_config(&self) -> Option<&Config> {
        match self {
            State::Playback { config, .. } => Some(config),
            State::Sync { .. } => None,
        }
    }

    fn get_name(&self) -> String {
        match self {
            State::Playback { name, .. } => name.to_string(),
            State::Sync { name, .. } => name.to_string(),
        }
    }

    // fn set_reset_context(&mut self, reset_context: bool) {
    //     todo!()
    // }
}
