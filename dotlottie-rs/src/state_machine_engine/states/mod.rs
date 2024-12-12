use std::{rc::Rc, sync::RwLock};

use serde::Deserialize;

use crate::{Config, DotLottiePlayerContainer, Layout, Mode};

use super::{actions::StateMachineActionError, transitions::Transition, StateMachineEngine};

use super::actions::{Action, ActionTrait};

#[derive(Debug, thiserror::Error)]
pub enum StatesError {
    #[error("Failed to parse JSON state machine definition")]
    ParsingError { reason: String },
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
    fn animation_id(&self) -> &str;
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
        animation_id: String,
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
        animation_id: Option<String>,
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
                animation_id,
                r#loop,
                autoplay,
                mode,
                speed,
                segment,
                background_color,
                use_frame_interpolation,
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
                    theme_id: "".to_string(),
                };

                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();

                    // Todo compare against currently loaded animation
                    if !animation_id.is_empty() && player_read.active_animation_id() != *animation_id
                    {
                        player_read.load_animation(animation_id, size.0, size.1);
                    }

                    player_read.set_config(playback_config);

                    /* Perform entry actions */
                    if let Some(actions) = entry_actions {
                        for action in actions {
                            let _ = action.execute(engine, player.clone(), false);
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
                animation_id,
                entry_actions,
                ..
            } => {
                if let Ok(player_read) = player.try_read() {
                    let size = player_read.size();

                    // Todo compare against currently loaded animation
                    if let Some(id) = animation_id {
                        player_read.load_animation(id, size.0, size.1);

                        // Perform entry actions
                        if let Some(actions) = entry_actions {
                            for action in actions {
                                let _ = action.execute(engine, player.clone(), false);
                            }
                        }
                    }
                }
            }
        }

        0
    }

    fn animation_id(&self) -> &str {
        match self {
            State::PlaybackState { animation_id, .. } => animation_id,
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
                        let _ = action.execute(engine, player.clone(), false);
                    }
                }
            }

            State::GlobalState { exit_actions, .. } => {
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let _ = action.execute(engine, player.clone(), false);
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
