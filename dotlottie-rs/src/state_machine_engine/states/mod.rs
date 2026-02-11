use std::ffi::CString;

use serde::Deserialize;

use crate::player::Mode;
use crate::DEFAULT_BACKGROUND_COLOR;

use super::{actions::StateMachineActionError, transitions::Transition, StateMachineEngine};

use super::actions::{Action, ActionTrait};

#[derive(Debug)]
pub enum StatesError {
    ParsingError,
}

pub trait StateTrait {
    fn enter(&self, engine: &mut StateMachineEngine) -> Result<(), StateMachineActionError>;
    fn exit(&self, engine: &mut StateMachineEngine) -> Result<(), StateMachineActionError>;
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
    fn enter(&self, engine: &mut StateMachineEngine) -> Result<(), StateMachineActionError> {
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
                let mut defined_mode = Mode::Forward;

                if let Some(new_mode) = mode {
                    match new_mode.as_str() {
                        "Forward" => defined_mode = Mode::Forward,
                        "Reverse" => defined_mode = Mode::Reverse,
                        "Bounce" => defined_mode = Mode::Bounce,
                        "ReverseBounce" => defined_mode = Mode::ReverseBounce,
                        _ => return Err(StateMachineActionError::ParsingError),
                    }
                }

                let size = engine.player.size();

                // Apply individual settings, preserving layout and use_frame_interpolation
                engine.player.set_mode(defined_mode);
                engine.player.set_loop(r#loop.unwrap_or(false));
                engine.player.set_loop_count(loop_count.unwrap_or(0));
                engine.player.set_speed(speed.unwrap_or(1.0));
                let _ = engine.player.set_background_color(Some(
                    background_color.unwrap_or(DEFAULT_BACKGROUND_COLOR),
                ));
                let _ = engine.player.set_segment(None);

                let marker_cstr = segment
                    .as_deref()
                    .map(CString::new)
                    .transpose()
                    .map_err(|_| StateMachineActionError::ParsingError)?;

                engine.player.set_marker(marker_cstr.as_deref());

                // set_autoplay must be called last as it triggers play/pause
                engine.player.set_autoplay(autoplay.unwrap_or(false));

                let Ok(anim_cstr) = CString::new(animation.as_str()) else {
                    return Err(StateMachineActionError::ParsingError);
                };

                if !animation.is_empty()
                    && engine.player.active_animation_id() != Some(&anim_cstr)
                    && engine.player.render().is_ok()
                {
                    let _ = engine.player.load_animation(&anim_cstr, size.0, size.1);
                }

                /* Perform entry actions */
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let _ = action.execute(engine, false, true);
                    }
                }

                if let Some(is_final) = r#final {
                    if *is_final {
                        engine.stop();
                    }
                }
            }
            State::GlobalState {
                animation,
                entry_actions,
                ..
            } => {
                let size = engine.player.size();

                let anim_cstr = animation
                    .as_deref()
                    .map(CString::new)
                    .transpose()
                    .map_err(|_| StateMachineActionError::ParsingError)?;

                if let Some(cstr) = anim_cstr {
                    if engine.player.active_animation_id() != Some(&cstr) {
                        let _ = engine.player.load_animation(&cstr, size.0, size.1);
                    }
                }

                // Perform entry actions
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let _ = action.execute(engine, false, true);
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

    fn exit(&self, engine: &mut StateMachineEngine) -> Result<(), StateMachineActionError> {
        match self {
            State::PlaybackState { exit_actions, .. } => {
                /* Perform exit actions */
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let _ = action.execute(engine, false, true);
                    }
                }
            }
            State::GlobalState { exit_actions, .. } => {
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let _ = action.execute(engine, false, true);
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
