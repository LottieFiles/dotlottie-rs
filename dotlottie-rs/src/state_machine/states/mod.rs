use std::{collections::HashMap, rc::Rc, sync::RwLock};

use crate::{Config, DotLottiePlayerContainer};

use super::{actions::Action, transitions::Transition};

pub trait StateTrait {
    fn execute(
        &self,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
        string_trigger: &HashMap<String, String>,
        bool_trigger: &HashMap<String, bool>,
        numeric_trigger: &HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> i32;
    fn get_animation_id(&self) -> Option<&String>;
    fn get_transitions(&self) -> &Vec<Transition>;
    fn add_transition(&mut self, transition: &Transition);
    fn get_config(&self) -> Option<&Config>;
    fn get_name(&self) -> String;
    fn get_type(&self) -> String;
}

#[derive(Clone, Debug)]
pub enum State {
    Playback {
        name: String,
        config: Config,
        animation_id: String,
        transitions: Vec<Transition>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
    Global {
        name: String,
        transitions: Vec<Transition>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
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
        _: &HashMap<String, f32>,
        _: &HashMap<String, String>,
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

            State::Global { .. } => {}
        }

        0
    }

    fn get_animation_id(&self) -> Option<&String> {
        match self {
            State::Playback { animation_id, .. } => Some(animation_id),
            State::Global { .. } => None,
        }
    }

    fn get_transitions(&self) -> &Vec<Transition> {
        match self {
            State::Playback { transitions, .. } => transitions,
            State::Global { transitions, .. } => transitions,
        }
    }

    fn add_transition(&mut self, transition: &Transition) {
        match self {
            State::Playback { transitions, .. } => transitions.push(transition.clone()),
            State::Global { transitions, .. } => transitions.push(transition.clone()),
        }
    }

    fn get_config(&self) -> Option<&Config> {
        match self {
            State::Playback { config, .. } => Some(config),
            State::Global { .. } => None,
        }
    }

    fn get_name(&self) -> String {
        match self {
            State::Playback { name, .. } => name.to_string(),
            State::Global { name, .. } => name.to_string(),
        }
    }

    fn get_type(&self) -> String {
        match self {
            State::Playback { .. } => "PlaybackState".to_string(),
            State::Global { .. } => "GlobalState".to_string(),
        }
    }
}
