use std::{rc::Rc, sync::RwLock};

use serde::Deserialize;

use crate::{dotlottie_player::Mode, Config, DotLottiePlayerContainer};

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
    ) -> Result<(), StateMachineActionError>;
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
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
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
                println!(
                    "[StateMachine] Entering PlaybackState: animation = '{}'",
                    animation
                );

                let default_config = Config::default();
                let mut defined_mode = default_config.mode;
                let mut defined_segment = default_config.marker;

                if let Some(new_mode) = mode {
                    println!("[StateMachine] Found mode override: '{}'", new_mode);
                    match new_mode.as_str() {
                        "Forward" => {
                            defined_mode = Mode::Forward;
                            println!("[StateMachine] Mode set to Forward");
                        }
                        "Reverse" => {
                            defined_mode = Mode::Reverse;
                            println!("[StateMachine] Mode set to Reverse");
                        }
                        "Bounce" => {
                            defined_mode = Mode::Bounce;
                            println!("[StateMachine] Mode set to Bounce");
                        }
                        "ReverseBounce" => {
                            defined_mode = Mode::ReverseBounce;
                            println!("[StateMachine] Mode set to ReverseBounce");
                        }
                        _ => {
                            println!("[StateMachine][ERROR] Unknown mode: '{}'", new_mode);
                            return Err(StateMachineActionError::ParsingError);
                        }
                    }
                }

                if let Some(new_segment) = segment {
                    println!("[StateMachine] Setting custom segment: '{}'", new_segment);
                    defined_segment = new_segment.clone();
                }

                if let Ok(player_read) = player.try_read() {
                    println!("[StateMachine] Acquired player read lock");
                    let size = player_read.size();
                    println!("[StateMachine] Player size = {:?}", size);
                    let current_config = player_read.config();

                    let uses_frame_interpolation = current_config.use_frame_interpolation;
                    let current_layout = current_config.layout.clone();

                    let playback_config = Config {
                        mode: defined_mode,
                        loop_animation: r#loop.unwrap_or(default_config.loop_animation),
                        loop_count: loop_count.unwrap_or(default_config.loop_count),
                        speed: speed.unwrap_or(default_config.speed),
                        use_frame_interpolation: uses_frame_interpolation,
                        autoplay: autoplay.unwrap_or(default_config.autoplay),
                        marker: defined_segment.clone(),
                        background_color: background_color
                            .unwrap_or(default_config.background_color),
                        layout: current_layout,
                        segment: [].to_vec(),
                        theme_id: current_config.theme_id,
                        state_machine_id: "".to_string(),
                        animation_id: "".to_string(),
                    };

                    println!(
                        "[StateMachine] Applying playback config: {:?}",
                        playback_config
                    );

                    // Set config first so that load_animation uses the preserved layout
                    player_read.set_config(playback_config);
                    println!("[StateMachine] Config is set on player");

                    if !animation.is_empty()
                        && player_read.active_animation_id() != *animation
                        && player_read.render()
                    {
                        println!(
                            "[StateMachine] Loading new animation '{}' (current was '{}')",
                            animation,
                            player_read.active_animation_id()
                        );
                        player_read.load_animation(animation, size.0, size.1);
                    } else {
                        if animation.is_empty() {
                            println!("[StateMachine] Animation name is empty, no reload required");
                        } else if player_read.active_animation_id() == *animation {
                            println!(
                                "[StateMachine] Animation '{}' already active, no reload",
                                animation
                            );
                        } else {
                            println!("[StateMachine] player.render() returned false, animation not loaded");
                        }
                    }

                    /* Perform entry actions */
                    if let Some(actions) = entry_actions {
                        println!("[StateMachine] Executing {} entry_actions", actions.len());
                        for action in actions {
                            println!("[StateMachine] Executing entry action: {:?}", action);
                            let result = action.execute(engine, player.clone(), false, true);
                            if let Err(e) = result {
                                println!("[StateMachine][ERROR] Action execution failed: {:?}", e);
                            }
                        }
                    } else {
                        println!("[StateMachine] No entry actions to perform");
                    }

                    if let Some(is_final) = r#final {
                        if *is_final {
                            println!("[StateMachine] This is a final state. Stopping engine.");
                            engine.stop();
                        }
                    }
                } else {
                    println!("[StateMachine][ERROR] Failed to acquire player read lock");
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
