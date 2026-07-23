use std::ffi::CString;

use crate::json::{array_of, opt, Value};
use crate::player::Mode;
use crate::state_machine::definition::dot_string;
use crate::string::{DotString, DotStringInterner};
use crate::Rgba;

use super::{actions::StateMachineActionError, transitions::Transition, StateMachineEngine};

use super::actions::{Action, ActionTrait};

pub trait StateTrait {
    fn enter(&self, engine: &mut StateMachineEngine) -> Result<(), StateMachineActionError>;
    fn exit(&self, engine: &mut StateMachineEngine) -> Result<(), StateMachineActionError>;
    fn animation(&self) -> &str;
    fn transitions(&self) -> &Vec<Transition>;
    fn entry_actions(&self) -> Option<&Vec<Action>>;
    fn exit_actions(&self) -> Option<&Vec<Action>>;
    fn name(&self) -> &DotString;
    fn get_type(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum State {
    PlaybackState {
        name: DotString,
        transitions: Vec<Transition>,
        animation: DotString,
        r#loop: Option<bool>,
        loop_count: Option<u32>,
        r#final: Option<bool>,
        autoplay: Option<bool>,
        mode: Option<Mode>,
        speed: Option<f32>,
        segment: Option<String>,
        background_color: Option<u32>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
    GlobalState {
        name: DotString,
        transitions: Vec<Transition>,
        entry_actions: Option<Vec<Action>>,
        exit_actions: Option<Vec<Action>>,
    },
}

pub(crate) fn state_from_json(v: &Value) -> Option<State> {
    let transitions = |v: &Value| -> Option<Vec<Transition>> {
        array_of(
            v.get("transitions")?,
            crate::state_machine::transitions::transition_from_json,
        )
    };
    let actions = |field: Option<&Value>| -> Option<Option<Vec<Action>>> {
        opt(field, |a| {
            array_of(a, crate::state_machine::actions::action_from_json)
        })
    };
    Some(match v.str_field("type")? {
        "PlaybackState" => State::PlaybackState {
            name: dot_string(v.get("name")?)?,
            transitions: transitions(v)?,
            animation: dot_string(v.get("animation")?)?,
            r#loop: opt(v.get("loop"), Value::as_bool)?,
            loop_count: opt(v.get("loopCount"), Value::as_u32)?,
            r#final: opt(v.get("final"), Value::as_bool)?,
            autoplay: opt(v.get("autoplay"), Value::as_bool)?,
            mode: opt(v.get("mode"), |m| Mode::from_json_str(m.as_str()?))?,
            speed: opt(v.get("speed"), Value::as_f32)?,
            segment: v.opt_str_field("segment")?,
            background_color: opt(v.get("backgroundColor"), Value::as_u32)?,
            entry_actions: actions(v.get("entryActions"))?,
            exit_actions: actions(v.get("exitActions"))?,
        },
        "GlobalState" => State::GlobalState {
            name: dot_string(v.get("name")?)?,
            transitions: transitions(v)?,
            entry_actions: actions(v.get("entryActions"))?,
            exit_actions: actions(v.get("exitActions"))?,
        },
        _ => return None,
    })
}

impl State {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        match self {
            State::PlaybackState {
                name,
                transitions,
                animation,
                entry_actions,
                exit_actions,
                ..
            } => {
                *name = interner.intern(name.as_str());
                *animation = interner.intern(animation.as_str());
                for t in transitions {
                    t.intern_identifiers(interner);
                }
                if let Some(actions) = entry_actions {
                    for a in actions {
                        a.intern_identifiers(interner);
                    }
                }
                if let Some(actions) = exit_actions {
                    for a in actions {
                        a.intern_identifiers(interner);
                    }
                }
            }
            State::GlobalState {
                name,
                transitions,
                entry_actions,
                exit_actions,
            } => {
                *name = interner.intern(name.as_str());
                for t in transitions {
                    t.intern_identifiers(interner);
                }
                if let Some(actions) = entry_actions {
                    for a in actions {
                        a.intern_identifiers(interner);
                    }
                }
                if let Some(actions) = exit_actions {
                    for a in actions {
                        a.intern_identifiers(interner);
                    }
                }
            }
        }
    }
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
                let defined_mode = match mode {
                    Some(m) => *m,
                    None => Mode::Forward,
                };

                // Apply individual settings, preserving layout and use_frame_interpolation
                engine.player.set_loop(r#loop.unwrap_or(false));
                engine.player.set_loop_count(loop_count.unwrap_or(0));
                engine.player.set_speed(speed.unwrap_or(1.0));
                let _ = engine
                    .player
                    .set_background(background_color.map_or(Rgba::TRANSPARENT, Rgba::from));
                let _ = engine.player.set_segment(None);
                engine.player.set_marker(None);

                if !animation.is_empty() {
                    let Ok(anim_cstr) = CString::new(animation.as_str()) else {
                        return Err(StateMachineActionError::ParsingError);
                    };

                    let needs_load = engine.player.animation_id() != Some(&anim_cstr);

                    if needs_load {
                        engine.player.set_autoplay(false);
                        // Clear any active theme before loading a different animation.
                        // load_animation() restores the saved theme after loading, but
                        // themes are animation-specific — the old theme's slot values
                        // may not exist in the new animation, causing render failures.
                        #[cfg(feature = "theming")]
                        {
                            let _ = engine.player.reset_theme();
                        }
                        let _ = engine.player.load_animation(&anim_cstr);
                    }
                }

                let marker_cstr = segment
                    .as_deref()
                    .map(CString::new)
                    .transpose()
                    .map_err(|_| StateMachineActionError::ParsingError)?;
                engine.player.set_marker(marker_cstr.as_deref());

                engine.player.set_mode(defined_mode);
                engine.player.set_autoplay(autoplay.unwrap_or(false));
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
            State::GlobalState { entry_actions, .. } => {
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

    fn name(&self) -> &DotString {
        match self {
            State::PlaybackState { name, .. } => name,
            State::GlobalState { name, .. } => name,
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
