use std::{rc::Rc, sync::RwLock};

use serde::Deserialize;

use crate::actions;
use crate::{dotlottie_player::Mode, Config, DotLottiePlayerContainer, Layout};

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
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> i32;
    fn exit(
        &self,
        engine: &mut StateMachineEngine,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
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
    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    fn enter(
        &self,
        engine: &mut StateMachineEngine,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> i32 {
        match self {
            State::PlaybackState {
                animation,
                r#loop,
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
                        _ => return 1,
                    }
                }

                if let Some(new_segment) = segment {
                    defined_segment = new_segment.clone();
                }

                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();
                    let uses_frame_interpolation = player_read.config().use_frame_interpolation;

                    let playback_config = Config {
                        mode: defined_mode,
                        loop_animation: r#loop.unwrap_or(default_config.loop_animation),
                        speed: speed.unwrap_or(default_config.speed),
                        use_frame_interpolation: uses_frame_interpolation,
                        autoplay: autoplay.unwrap_or(default_config.autoplay),
                        marker: defined_segment,
                        background_color: background_color
                            .unwrap_or(default_config.background_color),
                        layout: Layout::default(),
                        segment: [].to_vec(),
                        theme_id: "".to_string(),
                        state_machine_id: "".to_string(),
                        animation_id: "".to_string(),
                    };

                    if !animation.is_empty()
                        && player_read.active_animation_id() != *animation
                        && player_read.render()
                    {
                        player_read.load_animation(animation, size.0, size.1);
                    }

                    player_read.set_config(playback_config);

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
            State::GlobalState {
                animation,
                entry_actions,
                ..
            } => {
                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();

                    if let Some(animation) = animation {
                        if player_read.active_animation_id() != *animation {
                            player_read.load_animation(animation, size.0, size.1);
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
        }

        0
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
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
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
