use std::sync::Arc;

use serde::Deserialize;

use crate::DotLottiePlayer;
use crate::{dotlottie_player::Mode, Config, Layout};

use super::{actions::StateMachineActionError, transitions::Transition, StateMachineEngine};

use super::actions::{Action, ActionTrait};

#[derive(Debug)]
pub enum StatesError {
    ParsingError,
}

pub trait StateTrait {
    fn enter(
        &self,
        engine: &mut StateMachineEngine,
        player: &Arc<DotLottiePlayer>,
    ) -> Result<(), StateMachineActionError>;
    fn exit(
        &self,
        engine: &mut StateMachineEngine,
        player: &Arc<DotLottiePlayer>,
    ) -> Result<(), StateMachineActionError>;
    fn animation(&self) -> &str;
    fn transitions(&self) -> &Vec<Transition>;
    fn entry_actions(&self) -> Option<&Vec<Action>>;
    fn exit_actions(&self) -> Option<&Vec<Action>>;
    fn name(&self) -> String;
    fn get_type(&self) -> String;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum State {
    PlaybackState {
        name: String,
        transitions: Vec<Transition>,
        animation: String,
        r#loop: Option<bool>,
        loop_count: Option<u32>,
        r#final: Option<bool>,
        autoplay: Option<bool>,
        mode: Option<String>,
        speed: Option<f32>,
        segment: Option<String>,
        background_color: Option<u32>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
    GlobalState {
        name: String,
        transitions: Vec<Transition>,
        animation: Option<String>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
}

impl StateTrait for State {
    fn enter(
        &self,
        engine: &mut StateMachineEngine,
        player: &Arc<DotLottiePlayer>,
    ) -> Result<(), StateMachineActionError> {
        match self {
            State::PlaybackState {
                animation,
                r#loop,
                loop_count,
                r#final,
                autoplay,
                mode,
                speed,
                segment,
                background_color,
                entry_actions,
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
                        _ => return Err(StateMachineActionError::ParsingError),
                    }
                }

                if let Some(new_segment) = segment {
                    defined_segment = new_segment.clone();
                }

                let size = player.size();
                let uses_frame_interpolation = player.config().use_frame_interpolation;

                let playback_config = Config {
                    mode: defined_mode,
                    loop_animation: r#loop.unwrap_or(default_config.loop_animation),
                    loop_count: loop_count.unwrap_or(default_config.loop_count),
                    speed: speed.unwrap_or(default_config.speed),
                    use_frame_interpolation: uses_frame_interpolation,
                    autoplay: autoplay.unwrap_or(default_config.autoplay),
                    marker: defined_segment,
                    background_color: background_color.unwrap_or(default_config.background_color),
                    layout: Layout::default(),
                    segment: [].to_vec(),
                    theme_id: "".to_string(),
                    state_machine_id: "".to_string(),
                    animation_id: "".to_string(),
                };
                if !animation.is_empty() && player.active_animation_id() != *animation {
                    player.load_animation(animation, size.0, size.1);
                }

                println!("Setting config..");
                player.set_config(playback_config);
                println!("Set config..");

                /* Perform entry actions */
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let _ = action.execute(engine, player.clone(), false, true);
                    }
                }

                if let Some(is_final) = r#final {
                    if *is_final {
                        engine.stop();
                    }
                }
                // } else {
                //     println!("Player read error in enter");
                // }
            }
            State::GlobalState {
                animation,
                entry_actions,
                ..
            } => {
                let size = player.size();

                if let Some(animation) = animation {
                    if player.active_animation_id() != *animation {
                        player.load_animation(animation, size.0, size.1);
                    }
                }

                // Perform entry actions
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let _ = action.execute(engine, player.clone(), false, true);
                    }
                }
            }
        }

        Ok(())
    }

    fn animation(&self) -> &str {
        match self {
            State::PlaybackState { animation, .. } => animation,
            State::GlobalState { .. } => "",
        }
    }

    fn transitions(&self) -> &Vec<Transition> {
        match self {
            State::PlaybackState { transitions, .. } => transitions,
            State::GlobalState { transitions, .. } => transitions,
        }
    }

    fn name(&self) -> String {
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

    fn exit(
        &self,
        engine: &mut StateMachineEngine,
        player: &Arc<DotLottiePlayer>,
    ) -> Result<(), StateMachineActionError> {
        match self {
            State::PlaybackState { exit_actions, .. } => {
                /* Perform exit actions */
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let _ = action.execute(engine, player.clone(), false, true);
                    }
                }
            }
            State::GlobalState { exit_actions, .. } => {
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let _ = action.execute(engine, player.clone(), false, true);
                    }
                }
            }
        }

        Ok(())
    }

    fn entry_actions(&self) -> Option<&Vec<Action>> {
        match self {
            State::PlaybackState { entry_actions, .. } => entry_actions.as_ref(),
            State::GlobalState { entry_actions, .. } => entry_actions.as_ref(),
        }
    }

    fn exit_actions(&self) -> Option<&Vec<Action>> {
        match self {
            State::PlaybackState { exit_actions, .. } => exit_actions.as_ref(),
            State::GlobalState { exit_actions, .. } => exit_actions.as_ref(),
        }
    }
}
