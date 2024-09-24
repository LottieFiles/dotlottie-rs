use std::{collections::HashMap, rc::Rc, sync::RwLock};

use serde::Deserialize;

use crate::{Config, DotLottiePlayerContainer, Layout, Mode};

use super::{
    actions::{self, Action, ActionTrait, StateMachineActionError},
    transitions::Transition,
};

#[derive(Debug, thiserror::Error)]
pub enum StatesError {
    #[error("Failed to parse JSON state machine definition")]
    ParsingError { reason: String },

    #[error("Failed to create StateMachineEngine")]
    CreationError { reason: String },
}

pub trait StateTrait {
    fn execute(
        &self,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
        string_trigger: &HashMap<String, String>,
        bool_trigger: &HashMap<String, bool>,
        numeric_trigger: &HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> i32;
    fn enter(
        &self,
        player: &Option<Rc<RwLock<DotLottiePlayerContainer>>>,
        string_trigger: &mut HashMap<String, String>,
        bool_trigger: &mut HashMap<String, bool>,
        numeric_trigger: &mut HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> Result<(), StateMachineActionError>;
    fn exit(
        &self,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
        string_trigger: &HashMap<String, String>,
        bool_trigger: &HashMap<String, bool>,
        numeric_trigger: &HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> i32;
    fn get_animation_id(&self) -> Option<String>;
    fn get_transitions(&self) -> &Vec<Transition>;
    // fn add_transition(&mut self, transition: &Transition);
    // fn get_config(&self) -> Option<&Config>;
    fn get_name(&self) -> String;
    fn get_type(&self) -> String;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum State {
    PlaybackState {
        name: String,
        transitions: Vec<Transition>,
        animation_id: Option<String>,
        r#loop: Option<bool>,
        autoplay: Option<bool>,
        mode: Option<String>,
        speed: Option<f32>,
        segment: Option<String>,
        background_color: Option<u32>,
        use_frame_interpolation: Option<bool>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
    GlobalState {
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
            State::PlaybackState {
                animation_id,
                r#loop,
                autoplay,
                mode,
                speed,
                segment,
                background_color,
                use_frame_interpolation,
                ..
            } => {
                let default_config = Config::default();
                let mut defined_mode = default_config.mode;
                let mut defined_segment = default_config.marker;

                if let Some(new_mode) = mode {
                    match new_mode.as_str() {
                        "Forward" => defined_mode = Mode::Forward,
                        "Reverse" => defined_mode = Mode::Reverse,
                        "Bounce" => defined_mode = Mode::Bounce,
                        "ReverseBounce" => defined_mode = Mode::ReverseBounce,
                        _ => return 1,
                    }
                }

                println!("🚨 Segment: {:?}", segment);

                if let Some(new_segment) = segment {
                    defined_segment = new_segment.clone();
                }

                let playback_config = Config {
                    mode: defined_mode,
                    loop_animation: r#loop.unwrap_or(default_config.loop_animation),
                    speed: speed.unwrap_or(default_config.speed),
                    use_frame_interpolation: use_frame_interpolation
                        .unwrap_or(default_config.use_frame_interpolation),
                    autoplay: autoplay.unwrap_or(default_config.autoplay),
                    marker: defined_segment,
                    background_color: background_color.unwrap_or(default_config.background_color),
                    layout: Layout::default(),
                    segment: [].to_vec(),
                };

                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();

                    // Tell player to load new animation
                    if let Some(anim_id) = animation_id {
                        player_read.load_animation(anim_id, size.0, size.1);
                    }

                    println!("🚨 Playback config: {:?}", playback_config);

                    player_read.set_config(playback_config);

                    if let Some(autoplay) = autoplay {
                        if *autoplay {
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
            }

            State::GlobalState { .. } => {}
        }

        0
    }

    fn get_animation_id(&self) -> Option<String> {
        match self {
            State::PlaybackState { animation_id, .. } => animation_id.clone(),
            State::GlobalState { .. } => None,
        }
    }

    fn get_transitions(&self) -> &Vec<Transition> {
        match self {
            State::PlaybackState { transitions, .. } => transitions,
            State::GlobalState { transitions, .. } => transitions,
        }
    }

    // fn get_config(&self) -> Option<&Config> {
    //     match self {
    //         State::PlaybackState { config, .. } => Some(config),
    //         State::GlobalState { .. } => None,
    //     }
    // }

    fn get_name(&self) -> String {
        match self {
            State::PlaybackState { name, .. } => name.to_string(),
            State::GlobalState { name, .. } => name.to_string(),
        }
    }

    fn get_type(&self) -> String {
        match self {
            State::PlaybackState { .. } => "PlaybackState".to_string(),
            State::GlobalState { .. } => "GlobalState".to_string(),
        }
    }

    fn enter(
        &self,
        player: &Option<Rc<RwLock<DotLottiePlayerContainer>>>,
        string_trigger: &mut HashMap<String, String>,
        bool_trigger: &mut HashMap<String, bool>,
        numeric_trigger: &mut HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> Result<(), StateMachineActionError> {
        match self {
            State::PlaybackState {
                entry_actions,
                animation_id,
                ..
            } => {
                /* Perform entry actions */
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let _ = action.execute(
                            player,
                            string_trigger,
                            bool_trigger,
                            numeric_trigger,
                            event_trigger,
                        );
                    }
                }

                /* If theres an animation id, load it. */
                if let Some(anim_id) = animation_id {
                    if let Some(player) = player {
                        if let Ok(player_read) = player.try_read() {
                            let size = player_read.size();

                            player_read.load_animation(anim_id, size.0, size.1);
                        }
                    }
                }
            }

            State::GlobalState { entry_actions, .. } => {
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let _ = action.execute(
                            player,
                            string_trigger,
                            bool_trigger,
                            numeric_trigger,
                            event_trigger,
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn exit(
        &self,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
        string_trigger: &HashMap<String, String>,
        bool_trigger: &HashMap<String, bool>,
        numeric_trigger: &HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> i32 {
        println!("🚨 Exit state not yet implemented");
        0
    }
}
