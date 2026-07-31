use core::result::Result::Ok;
use std::ffi::{CStr, CString};

use rustc_hash::FxHashSet;

use crate::string::{DotString, DotStringInterner};

pub(crate) const GLOBAL_INPUT_PREFIX: char = '@';
pub(crate) const ELAPSED_TIME: &str = "@elapsedTime";

const DEFAULT_RNG_SEED: u64 = 0x853c_49e6_748f_ea9b;

pub mod actions;
pub mod drag_and_drop;
pub mod errors;
pub mod events;
pub mod inputs;
pub mod interactions;
pub mod path_drag;
pub mod security;
pub mod state_machine;
pub mod state_slots;
pub mod states;
pub mod transitions;

use actions::open_url_policy::OpenUrlPolicy;
use actions::{Action, ActionTrait};
use inputs::{Input, InputManager, InputValue};
use interactions::InteractionTrait;
use state_machine::StateMachine;
use states::StateTrait;
use transitions::guard::{Guard, GuardTrait};
use transitions::{Transition, TransitionTrait};

use crate::state_machine::StringNumberBool;

use crate::actions::whitelist::Whitelist;
use crate::event_queue::EventQueue;
use crate::state_machine_engine::events::{StateMachineEvent, StateMachineInternalEvent};
use crate::state_machine_engine::interactions::Interaction;
use crate::{
    event_type_name, state_machine_state_check_pipeline, CompletionEvent, EventName, Layout, Mode,
    Player, Point, PointerEvent, Rgba, Segment, StateMachineEngineSecurityError,
};

use self::drag_and_drop::{DndPhase, DndRuntime};
use self::path_drag::{PathDragRuntime, PathSnap};
use self::state_machine::state_machine_parse;
use self::state_slots::{SlotLerp, StateSlot};
use self::{events::Event, states::State};

use crate::lottie_renderer::{PositionSlot, SlotType};

#[derive(PartialEq, Debug)]
pub enum StateMachineEngineStatus {
    Running,
    Tweening,
    Stopped,
}

#[derive(Debug)]
pub enum StateMachineEngineError {
    ParsingError(String),
    CreationError,
    FireEventError,
    InfiniteLoopError,
    NotRunningError,
    SetStateError,
    SecurityCheckErrorMultipleGuardlessTransitions,
    SecurityCheckErrorDuplicateStateName,
}

struct PointerData {
    // DotString so comparisons against interned interaction layer names
    // hit the Arc::ptr_eq fast path.
    curr_entered_layer: DotString,
    listened_layers: Vec<(DotString, &'static str)>,
    most_recent_event: Option<Event>,
    pointer_x: f32,
    pointer_y: f32,
}

impl Default for PointerData {
    fn default() -> PointerData {
        PointerData {
            curr_entered_layer: DotString::empty(),
            listened_layers: Vec::new(),
            most_recent_event: None,
            pointer_x: 0.0,
            pointer_y: 0.0,
        }
    }
}

pub struct StateMachineEngine<'a> {
    // For restoring the player config after state machine is stopped
    cached_mode: Mode,
    cached_speed: f32,
    cached_loop_animation: bool,
    cached_loop_count: u32,
    cached_autoplay: bool,
    cached_use_frame_interpolation: bool,
    cached_background: Rgba,
    cached_segment: Option<Segment>,
    cached_marker: Option<CString>,
    cached_layout: Layout,

    /* We keep references to the StateMachine's States. */
    /* This prevents duplicating the data inside the engine. */
    pub global_state: Option<State>,
    pub current_state: Option<State>,

    pub status: StateMachineEngineStatus,

    // Open url policy configurations
    pub open_url_requires_user_interaction: bool,
    pub open_url_whitelist: Whitelist,

    pub player: &'a mut Player,

    pub inputs: InputManager,
    curr_event: Option<DotString>,

    // PointerEnter/PointerExit management
    pointer_management: PointerData,

    // Event queues
    pub event_queue: EventQueue<StateMachineEvent, 32>,
    pub internal_event_queue: EventQueue<StateMachineInternalEvent, 8>,

    // Holds current event during polling from C API
    pub current_event: Option<StateMachineEvent>,
    pub current_internal_event: Option<StateMachineInternalEvent>,

    pub(crate) str_interner: DotStringInterner,

    state_machine: StateMachine,

    state_history: Vec<DotString>,
    max_cycle_count: usize,
    current_cycle_count: usize,
    action_mutated_inputs: bool,

    // The state to target once blending has finished
    tween_transition_target_state: Option<State>,
    tween_target_frame: Option<f32>,

    // ── State-declared slots (prototype) ─────────────────────────────
    // Pre-overlay values to restore when the declaring state releases a slot.
    slot_overlay_bases: std::collections::BTreeMap<String, SlotType>,
    // The current state's declared slots (live-bound while current).
    active_state_slots: Vec<StateSlot>,
    // In-flight per-slot interpolations during a Tweened transition.
    slot_lerps: Vec<SlotLerp>,
    // Non-interpolable releases to restore when the tween completes.
    slot_snap_releases: Vec<(String, SlotType)>,

    // DragAndDrop gesture runtimes (one per DragAndDrop interaction).
    dnd_runtimes: Vec<DndRuntime>,

    // PathDrag gesture runtimes (one per PathDrag interaction).
    pathdrag_runtimes: Vec<PathDragRuntime>,

    elapsed_time: f32,
    elapsed_time_states: FxHashSet<DotString>,
    elapsed_time_in_global: bool,

    rng: oorandom::Rand32,
    rng_seed: u64,
}

impl<'a> StateMachineEngine<'a> {
    pub fn new(
        state_machine_definition: &str,
        player: &'a mut Player,
        max_cycle_count: Option<usize>,
    ) -> Result<StateMachineEngine<'a>, StateMachineEngineError> {
        Self::from_definition(state_machine_definition, player, max_cycle_count)
    }

    /// Poll for the next state machine event
    ///
    /// Returns Some(event) if an event is available, None if the queue is empty.
    /// Events are removed from the queue when polled.
    pub fn poll_event(&mut self) -> Option<StateMachineEvent> {
        self.event_queue.poll()
    }

    /// Poll for the next internal state machine event
    ///
    /// Returns Some(event) if an event is available, None if the queue is empty.
    /// Internal events are for framework use only.
    pub fn poll_internal_event(&mut self) -> Option<StateMachineInternalEvent> {
        self.internal_event_queue.poll()
    }

    // key: The key of the input
    // value: The value to set the input to
    // run_pipeline: If true, the pipeline will be run after setting the input. This is most likely false if called from an action or during initialization.
    // called_from_action: If true, the input was set from an action. We need this so that action_mutated_inputs is correctly set.
    pub fn set_numeric_input(
        &mut self,
        key: &str,
        value: f32,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<f32> {
        if key.starts_with(GLOBAL_INPUT_PREFIX) {
            return None;
        }

        // Modifying triggers whilst tweening isn't allowed
        if self.status == StateMachineEngineStatus::Tweening {
            return None;
        }

        let ret = self.inputs.set_numeric(key, value);

        if let Some(old_value) = ret {
            self.observe_numeric_input_value_change(key, old_value, value);
            // Live-bound state slots referencing this input re-apply.
            self.reapply_bound_state_slots(key);
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }

        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }

        ret
    }

    pub fn get_numeric_input(&self, key: &str) -> Option<f32> {
        if key == ELAPSED_TIME {
            return Some(self.elapsed_time);
        }
        self.inputs.get_numeric(key)
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rng_seed = seed;
        self.rng = oorandom::Rand32::new(seed);
    }

    pub(crate) fn next_random(&mut self) -> f32 {
        self.rng.rand_float()
    }

    pub fn set_string_input(
        &mut self,
        key: &str,
        value: &str,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<String> {
        if key.starts_with(GLOBAL_INPUT_PREFIX) {
            return None;
        }

        // Modifying triggers whilst tweening isn't allowed
        if self.status == StateMachineEngineStatus::Tweening {
            return None;
        }

        let ret = self.inputs.set_string(key, value.to_string());

        if let Some(ref old_value) = ret {
            self.observe_string_input_value_change(key, old_value, value);
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }

        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }

        ret
    }

    pub fn get_string_input(&self, key: &str) -> Option<String> {
        self.inputs.get_string(key).map(Into::into)
    }

    pub fn set_boolean_input(
        &mut self,
        key: &str,
        value: bool,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<bool> {
        if key.starts_with(GLOBAL_INPUT_PREFIX) {
            return None;
        }

        // Modifying triggers whilst tweening isn't allowed
        if self.status == StateMachineEngineStatus::Tweening {
            return None;
        }

        let ret = self.inputs.set_boolean(key, value);

        if let Some(old_value) = ret {
            self.observe_boolean_input_value_change(key, old_value, value);
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }

        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }

        ret
    }

    pub fn get_boolean_input(&self, key: &str) -> Option<bool> {
        self.inputs.get_boolean(key)
    }

    pub fn reset_input(&mut self, key: &str, run_pipeline: bool, called_from_action: bool) {
        if self.status != StateMachineEngineStatus::Running {
            return;
        }

        if key.starts_with(GLOBAL_INPUT_PREFIX) {
            return;
        }

        if let Some((old, new)) = self.inputs.reset(key) {
            match (old, new) {
                (InputValue::Numeric(old), InputValue::Numeric(new)) => {
                    self.observe_numeric_input_value_change(key, old, new);
                    // Live-bound state slots referencing this input re-apply.
                    self.reapply_bound_state_slots(key);
                }
                (InputValue::String(old), InputValue::String(new)) => {
                    self.observe_string_input_value_change(key, &old, &new);
                }
                (InputValue::Boolean(old), InputValue::Boolean(new)) => {
                    self.observe_boolean_input_value_change(key, old, new);
                }
                _ => {}
            }
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }

        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }
    }

    pub fn fire(&mut self, event: &str, run_pipeline: bool) -> Result<(), StateMachineEngineError> {
        if self.inputs.get_event(event).is_some() {
            self.observe_on_input_fired(event);
            self.curr_event = Some(self.str_interner.intern(event));

            // Run pipeline is always false if called from an action
            if run_pipeline {
                let _ = self.run_current_state_pipeline();
            }

            return Ok(());
        }

        Err(StateMachineEngineError::FireEventError)
    }

    // Parses the JSON of the state machine definition and creates the states and transitions
    // Previously called create_state_machine
    pub fn from_definition(
        sm_definition: &str,
        player: &'a mut Player,
        max_cycle_count: Option<usize>,
    ) -> Result<StateMachineEngine<'a>, StateMachineEngineError> {
        let parsed_state_machine = state_machine_parse(sm_definition);
        let mut new_state_machine = StateMachineEngine {
            cached_mode: player.mode(),
            cached_speed: player.speed(),
            cached_loop_animation: player.loop_animation(),
            cached_loop_count: player.current_loop_count(),
            cached_autoplay: player.autoplay(),
            cached_use_frame_interpolation: player.use_frame_interpolation(),
            cached_background: player.background(),
            cached_segment: player.segment().ok(),
            cached_marker: player.active_marker().map(CStr::to_owned),
            cached_layout: player.layout(),
            player, // `player` Moved. Don't use after this point
            global_state: None,
            state_machine: StateMachine::default(),
            current_state: None,
            open_url_requires_user_interaction: false,
            open_url_whitelist: Whitelist::new(),
            inputs: InputManager::new(),
            curr_event: None,
            pointer_management: PointerData::default(),
            status: StateMachineEngineStatus::Stopped,
            event_queue: EventQueue::new(),
            internal_event_queue: EventQueue::new(),
            current_event: None,
            current_internal_event: None,
            str_interner: DotStringInterner::new(),
            state_history: Vec::new(),
            max_cycle_count: max_cycle_count.unwrap_or(20),
            current_cycle_count: 0,
            action_mutated_inputs: false,
            tween_transition_target_state: None,
            tween_target_frame: None,
            slot_overlay_bases: std::collections::BTreeMap::new(),
            active_state_slots: Vec::new(),
            slot_lerps: Vec::new(),
            slot_snap_releases: Vec::new(),
            dnd_runtimes: Vec::new(),
            pathdrag_runtimes: Vec::new(),
            elapsed_time: 0.0,
            elapsed_time_states: FxHashSet::default(),
            elapsed_time_in_global: false,
            rng: oorandom::Rand32::new(DEFAULT_RNG_SEED),
            rng_seed: DEFAULT_RNG_SEED,
        };

        if parsed_state_machine.is_err() {
            let message = match parsed_state_machine.err() {
                Some(e) => format!("Parsing error: {e:?}"),
                None => "Parsing error: Unknown error".to_string(),
            };

            new_state_machine.observe_on_error(message.as_str());

            return Err(StateMachineEngineError::ParsingError(message));
        }

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                /* Build all input variables into hashmaps for easier use */
                if let Some(inputs) = &parsed_state_machine.inputs {
                    for input in inputs {
                        match input {
                            Input::Numeric { name, value } => {
                                new_state_machine.inputs.set_initial_numeric(name, *value);
                            }
                            Input::String { name, value } => {
                                new_state_machine.inputs.set_initial_string(name, value);
                            }
                            Input::Boolean { name, value } => {
                                new_state_machine.inputs.set_initial_boolean(name, *value);
                            }
                            Input::Event { name } => {
                                new_state_machine.inputs.set_initial_event(name);
                            }
                        }
                    }
                }

                /*
                   Set the reference to the global state so that we can easily
                   Access it when evaluating transitions
                */
                for state in &parsed_state_machine.states {
                    if let State::GlobalState { .. } = state {
                        new_state_machine.global_state = Some(state.clone());
                    }
                }

                new_state_machine.state_machine = parsed_state_machine;

                // Canonicalize all identifiers so runtime comparisons hit ptr_eq.
                new_state_machine
                    .state_machine
                    .intern_identifiers(&mut new_state_machine.str_interner);

                let (states, in_global) =
                    compute_elapsed_time_states(&new_state_machine.state_machine);
                new_state_machine.elapsed_time_states = states;
                new_state_machine.elapsed_time_in_global = in_global;

                new_state_machine.init_listened_layers();

                // Run the security check pipeline
                let check_report = Self::security_check_pipeline(&new_state_machine);

                match check_report {
                    Ok(_) => {}
                    Err(error) => {
                        let message = format!("Load: {error:?}");

                        new_state_machine.observe_on_error(message.as_str());

                        return Err(StateMachineEngineError::CreationError);
                    }
                }

                Ok(new_state_machine)
            }
            Err(_error) => Err(StateMachineEngineError::CreationError),
        }
    }

    fn security_check_pipeline(
        state_machine: &StateMachineEngine,
    ) -> Result<(), StateMachineEngineSecurityError> {
        state_machine_state_check_pipeline(state_machine)
    }

    pub fn start(&mut self, open_url: &OpenUrlPolicy) -> Result<(), crate::PlayerError> {
        // Reset to first frame
        let _ = self.player.stop();
        self.player.set_mode(Mode::Forward);
        self.player.set_speed(1.0);
        self.player.set_loop(false);
        self.player.set_loop_count(0);
        self.player.set_autoplay(false);

        // Start can still be called even if load failed. If load failed initial and states will be empty.
        if self.state_machine.initial.is_empty() || self.state_machine.states.is_empty() {
            return Err(crate::PlayerError::Unknown);
        }

        self.open_url_requires_user_interaction = open_url.require_user_interaction;

        if !open_url.whitelist.is_empty() {
            let mut whitelist = Whitelist::new();

            // Add patterns to whitelist
            for entry in &open_url.whitelist {
                let _ = whitelist.add(entry);
            }

            self.open_url_whitelist = whitelist;
        }

        self.elapsed_time = 0.0;
        self.rng = oorandom::Rand32::new(self.rng_seed);

        // Build DragAndDrop gesture runtimes from their interactions.
        self.dnd_runtimes = self
            .state_machine
            .interactions
            .iter()
            .flatten()
            .filter_map(DndRuntime::from_interaction)
            .collect();

        // Build PathDrag gesture runtimes + their arc-length sample tables
        // (paths come from the load-time extraction of authored beziers).
        self.pathdrag_runtimes = self
            .state_machine
            .interactions
            .iter()
            .flatten()
            .filter_map(PathDragRuntime::from_interaction)
            .collect();
        let mut runtimes = std::mem::take(&mut self.pathdrag_runtimes);
        for rt in &mut runtimes {
            if let Some(path) = self.player.layer_path(&rt.path_layer_name) {
                rt.build_samples(&path);
            }
        }
        self.pathdrag_runtimes = runtimes;

        let initial = &self.state_machine.initial.clone();

        let err = self.set_current_state(initial, None, false);
        match err {
            Ok(_) => {}
            Err(error) => {
                let message = format!("Error setting initial state: {error:?}");

                self.observe_on_error(message.as_str());

                return Err(crate::PlayerError::Unknown);
            }
        }

        if self.status == StateMachineEngineStatus::Running {
            return Ok(());
        }

        self.observe_on_start();

        self.status = StateMachineEngineStatus::Running;

        let _ = self.run_current_state_pipeline();

        Ok(())
    }

    pub fn stop(&mut self) {
        self.status = StateMachineEngineStatus::Stopped;

        self.observe_on_stop();

        self.player.set_mode(self.cached_mode);
        self.player.set_speed(self.cached_speed);
        self.player.set_loop(self.cached_loop_animation);
        self.player.set_loop_count(self.cached_loop_count);
        self.player
            .set_use_frame_interpolation(self.cached_use_frame_interpolation);
        let _ = self.player.set_background(self.cached_background);
        let _ = self.player.set_segment(self.cached_segment);
        self.player.set_marker(self.cached_marker.as_deref());
        let _ = self.player.set_layout(self.cached_layout);
        self.player.set_autoplay(self.cached_autoplay);
    }

    /// For external use only.
    /// `mut self` here drops state_machine_engine which releases the borrow of `dotlottie_player`
    pub fn release(mut self) {
        if self.status != StateMachineEngineStatus::Stopped {
            self.stop();
        }
    }

    pub fn status(&self) -> String {
        match self.status {
            StateMachineEngineStatus::Running => "Running".to_string(),
            StateMachineEngineStatus::Tweening => "Tweening".to_string(),
            StateMachineEngineStatus::Stopped => "Stopped".to_string(),
        }
    }

    pub fn get_current_state(&self) -> Option<State> {
        self.current_state.clone()
    }

    pub fn interactions<'b>(
        &'b self,
        event_type_filter: Option<&'b str>,
    ) -> impl Iterator<Item = &'b Interaction> {
        self.state_machine
            .interactions
            .iter()
            .flatten()
            .filter(move |interaction| {
                event_type_filter.is_none_or(|f| f == interaction.type_name())
            })
    }

    pub fn framework_setup(&self) -> Vec<String> {
        let mut interaction_types = vec![];

        for interaction in self.interactions(None) {
            match interaction {
                crate::interactions::Interaction::PointerUp { .. } => {
                    interaction_types.push("PointerUp".to_string())
                }
                crate::interactions::Interaction::PointerDown { .. } => {
                    interaction_types.push("PointerDown".to_string())
                }
                crate::interactions::Interaction::PointerEnter { .. } => {
                    // In case framework self detects pointer entering layers, push pointerExit
                    interaction_types.push("PointerEnter".to_string());
                    // We push PointerMove too so that we can do hit detection instead of the framework
                    interaction_types.push("PointerMove".to_string());
                }
                crate::interactions::Interaction::PointerMove { .. } => {
                    interaction_types.push("PointerMove".to_string())
                }
                crate::interactions::Interaction::PointerExit { .. } => {
                    // In case framework self detects pointer exiting layers, push pointerExit
                    interaction_types.push("PointerExit".to_string());
                    // We push PointerMove too so that we can do hit detection instead of the framework
                    interaction_types.push("PointerMove".to_string());
                }
                crate::interactions::Interaction::OnComplete { .. } => {
                    interaction_types.push("OnComplete".to_string())
                }
                crate::interactions::Interaction::OnLoopComplete { .. } => {
                    interaction_types.push("OnLoopComplete".to_string())
                }
                crate::interactions::Interaction::Click { .. } => {
                    interaction_types.push("Click".to_string());
                }
                crate::interactions::Interaction::DragAndDrop { .. } => {
                    interaction_types.push("PointerDown".to_string());
                    interaction_types.push("PointerMove".to_string());
                    interaction_types.push("PointerUp".to_string());
                }
            }
        }

        interaction_types.sort();
        interaction_types.dedup();
        interaction_types
    }

    fn init_listened_layers(&mut self) {
        let interactions: Vec<_> = self.interactions(None).collect();

        let mut all_listened_layers: Vec<(DotString, &'static str)> = vec![];

        for interaction in interactions {
            match interaction {
                Interaction::PointerEnter {
                    layer_name: Some(layer),
                    ..
                } => {
                    all_listened_layers.push((layer.clone(), event_type_name!(PointerEnter)));
                }
                Interaction::PointerExit {
                    layer_name: Some(layer),
                    ..
                } => all_listened_layers.push((layer.clone(), event_type_name!(PointerExit))),
                Interaction::PointerUp {
                    layer_name: Some(layer),
                    ..
                } => all_listened_layers.push((layer.clone(), event_type_name!(PointerUp))),
                Interaction::PointerDown {
                    layer_name: Some(layer),
                    ..
                } => all_listened_layers.push((layer.clone(), event_type_name!(PointerDown))),
                _ => {}
            }
        }

        self.pointer_management.listened_layers = all_listened_layers;
    }

    fn get_state(&self, state_name: &str) -> Option<State> {
        if let Some(global_state) = &self.global_state {
            if global_state.name() == state_name {
                return Some(global_state.clone());
            }
        }

        for state in self.state_machine.states.iter() {
            if state.name() == state_name {
                return Some(state.clone());
            }
        }

        None
    }

    pub fn resume_from_tweening(&mut self) {
        if self.status != StateMachineEngineStatus::Tweening {
            return;
        }

        self.status = StateMachineEngineStatus::Running;

        // Snap in-flight slot interpolations to their exact targets and
        // restore non-interpolable releases.
        self.finalize_state_slot_tween();

        if let Some(target_state) = &self.tween_transition_target_state {
            // Assign the new state to the current_state
            self.current_state = Some(target_state.clone());

            self.tween_transition_target_state = None;

            // Emit transtion occured event
            self.observe_on_state_entered(&self.get_current_state_name());

            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();

            // Now use the extracted information
            if let Some(state) = state {
                let _ = state.enter(self);

                if let Some(target_frame) = self.tween_target_frame.take() {
                    self.player.sync_tween_frame(target_frame);
                }

                // Don't forget to put things back
                // new_state becomes the current state
                self.current_state = Some(state);
            }
        }
    }

    // ── State-declared slots (prototype) ─────────────────────────────

    /// Apply one declared slot entry at the given resolved components,
    /// saving the pre-overlay base value the first time the overlay covers
    /// the slot. Unknown slot IDs and type mismatches are silent no-ops.
    fn apply_state_slot_entry(&mut self, entry: &StateSlot, comps: &[f32]) {
        let slot_id = entry.slot_id.as_str();

        let Some(current) = self.player.slot_value(slot_id) else {
            return;
        };
        if !state_slots::type_compatible(entry.slot_type, &current) {
            return;
        }

        if !self.slot_overlay_bases.contains_key(slot_id) {
            self.slot_overlay_bases.insert(slot_id.to_string(), current);
        }

        if let Some(slot) = state_slots::build_slot(entry.slot_type, comps) {
            let _ = self.player.set_slot_value(slot_id, slot);
        }
    }

    /// Instantly apply a state's declared slots and make them the active
    /// overlay (live-bound while the state is current).
    fn apply_state_slots(&mut self, slots: &[StateSlot]) {
        for entry in slots {
            if let Some(comps) = entry.resolve(self) {
                self.apply_state_slot_entry(entry, &comps);
            }
        }
        self.active_state_slots = slots.to_vec();
    }

    /// Release overlay slots the incoming state does not redeclare,
    /// restoring their saved base values (theme value or authored default).
    /// Redeclared slots keep their original saved base.
    fn release_state_slots(&mut self, next: Option<&State>) {
        if self.active_state_slots.is_empty() {
            return;
        }

        let outgoing = std::mem::take(&mut self.active_state_slots);
        for entry in &outgoing {
            let redeclared = next
                .and_then(|s| s.state_slots())
                .is_some_and(|slots| slots.iter().any(|n| n.slot_id == entry.slot_id));
            if redeclared {
                continue;
            }

            if let Some(base) = self.slot_overlay_bases.remove(entry.slot_id.as_str()) {
                let _ = self.player.set_slot_value(entry.slot_id.as_str(), base);
            }
        }
    }

    /// Drop all overlay bookkeeping without restoring anything — used when
    /// the animation changes: the outgoing animation's slot state does not
    /// survive the load, and the renderer reseeds slots from the new
    /// animation's authored defaults.
    fn reset_state_slot_bookkeeping(&mut self) {
        self.slot_overlay_bases.clear();
        self.active_state_slots.clear();
        self.slot_lerps.clear();
        self.slot_snap_releases.clear();
    }

    /// Set up per-slot interpolations for a `Tweened` transition into
    /// `target` (same-animation only). References sample at tween start;
    /// the target's declarations become the active overlay immediately
    /// (input writes are rejected while tweening, so bindings cannot fire
    /// mid-tween).
    fn prepare_state_slot_tween(&mut self, target: &State) {
        self.slot_lerps.clear();
        self.slot_snap_releases.clear();

        let target_slots: Vec<StateSlot> = target.state_slots().cloned().unwrap_or_default();

        // Releases: outgoing overlay entries the target doesn't redeclare
        // tween back to their base values (snap at completion when either
        // endpoint isn't a static interpolable value).
        let outgoing = std::mem::take(&mut self.active_state_slots);
        for entry in &outgoing {
            if target_slots.iter().any(|n| n.slot_id == entry.slot_id) {
                continue;
            }
            let slot_id = entry.slot_id.as_str();
            let Some(base) = self.slot_overlay_bases.remove(slot_id) else {
                continue;
            };

            let from = self
                .player
                .slot_value(slot_id)
                .as_ref()
                .and_then(state_slots::static_components);
            let to = state_slots::static_components(&base);

            match (from, to) {
                (Some(from), Some(to)) if from.len() == to.len() => {
                    self.slot_lerps.push(SlotLerp {
                        slot_id: slot_id.to_string(),
                        slot_type: entry.slot_type,
                        from,
                        to,
                    });
                }
                _ => self.slot_snap_releases.push((slot_id.to_string(), base)),
            }
        }

        // Targets: declared slots tween from their current values. Entries
        // whose current value isn't a static interpolable (e.g. a keyframed
        // authored value) apply instantly at tween start.
        for entry in &target_slots {
            let Some(to) = entry.resolve(self) else {
                continue;
            };
            let slot_id = entry.slot_id.as_str();
            let Some(current) = self.player.slot_value(slot_id) else {
                continue;
            };
            if !state_slots::type_compatible(entry.slot_type, &current) {
                continue;
            }

            if !self.slot_overlay_bases.contains_key(slot_id) {
                self.slot_overlay_bases
                    .insert(slot_id.to_string(), current.clone());
            }

            match state_slots::static_components(&current) {
                Some(from) if from.len() == to.len() => {
                    self.slot_lerps.push(SlotLerp {
                        slot_id: slot_id.to_string(),
                        slot_type: entry.slot_type,
                        from,
                        to,
                    });
                }
                _ => {
                    if let Some(slot) = state_slots::build_slot(entry.slot_type, &to) {
                        let _ = self.player.set_slot_value(slot_id, slot);
                    }
                }
            }
        }

        self.active_state_slots = target_slots;
    }

    /// Advance in-flight slot interpolations to the given eased progress.
    fn apply_slot_lerps(&mut self, progress: f32) {
        if self.slot_lerps.is_empty() {
            return;
        }

        let lerps = std::mem::take(&mut self.slot_lerps);
        for lerp in &lerps {
            let comps = state_slots::lerp_components(&lerp.from, &lerp.to, progress);
            if let Some(slot) = state_slots::build_slot(lerp.slot_type, &comps) {
                let _ = self.player.set_slot_value(&lerp.slot_id, slot);
            }
        }
        self.slot_lerps = lerps;
    }

    /// Complete slot interpolation: snap lerps to their exact targets and
    /// restore non-interpolable releases.
    fn finalize_state_slot_tween(&mut self) {
        let lerps = std::mem::take(&mut self.slot_lerps);
        for lerp in &lerps {
            if let Some(slot) = state_slots::build_slot(lerp.slot_type, &lerp.to) {
                let _ = self.player.set_slot_value(&lerp.slot_id, slot);
            }
        }

        let snaps = std::mem::take(&mut self.slot_snap_releases);
        for (slot_id, base) in snaps {
            let _ = self.player.set_slot_value(&slot_id, base);
        }
    }

    /// Re-apply live-bound declarations that reference `input_name`.
    fn reapply_bound_state_slots(&mut self, input_name: &str) {
        if self.status != StateMachineEngineStatus::Running || self.active_state_slots.is_empty() {
            return;
        }

        let entries = std::mem::take(&mut self.active_state_slots);
        for entry in &entries {
            if entry.referenced_inputs().any(|name| name == input_name) {
                if let Some(comps) = entry.resolve(self) {
                    self.apply_state_slot_entry(entry, &comps);
                }
            }
        }
        self.active_state_slots = entries;
    }

    // ── DragAndDrop interaction runtime (prototype) ───────────────────

    /// Current position components of a slot, if it holds a static 2D value.
    fn dnd_slot_position(&self, slot_id: &str) -> Option<[f32; 2]> {
        let slot = self.player.slot_value(slot_id)?;
        let comps = state_slots::static_components(&slot)?;
        (comps.len() == 2).then(|| [comps[0], comps[1]])
    }

    /// Canvas-pixel pointer -> COMPOSITION units, the space of slots,
    /// layer transforms, and extracted paths. All gesture math runs in
    /// comp units; only scene queries (hit tests) take raw canvas pixels.
    fn to_comp(&self, x: f32, y: f32) -> [f32; 2] {
        self.player.canvas_to_comp(x, y)
    }

    /// A layer's current transform position (matrix translation, comp
    /// units, parent chain composed) — exact when the anchor is zero.
    fn layer_position(&self, layer_name: &str) -> Option<[f32; 2]> {
        let m = self.player.layer_transform(layer_name)?;
        Some([m[2], m[5]])
    }

    /// Center of a layer's current rendered bounds, converted to comp
    /// units (as of the last render).
    fn layer_center(&self, layer_name: &str) -> Option<[f32; 2]> {
        let obb = self.player.layer_bounds(layer_name)?;
        Some(self.player.canvas_to_comp(
            (obb[0].x + obb[1].x + obb[2].x + obb[3].x) / 4.0,
            (obb[0].y + obb[1].y + obb[2].y + obb[3].y) / 4.0,
        ))
    }

    /// A layer's rendered OBB corners converted to comp units.
    fn layer_bounds_comp(&self, layer_name: &str) -> Option<[[f32; 2]; 4]> {
        let obb = self.player.layer_bounds(layer_name)?;
        Some([
            self.player.canvas_to_comp(obb[0].x, obb[0].y),
            self.player.canvas_to_comp(obb[1].x, obb[1].y),
            self.player.canvas_to_comp(obb[2].x, obb[2].y),
            self.player.canvas_to_comp(obb[3].x, obb[3].y),
        ])
    }

    /// Clamp a held object's transform position so its VISUAL rect stays
    /// inside the boundary layer's current rendered bounds. Works on the
    /// OBB's own axes (projections onto its edge vectors), so rotated
    /// boundaries clamp correctly too. No boundary layer or degenerate
    /// bounds = no constraint.
    fn dnd_clamp_to_boundary(&self, rt: &DndRuntime, pos: [f32; 2]) -> [f32; 2] {
        let Some(boundary) = rt.boundary.as_deref() else {
            return pos;
        };
        let Some(obb) = self.layer_bounds_comp(boundary) else {
            return pos;
        };

        let (e1x, e1y) = (obb[1][0] - obb[0][0], obb[1][1] - obb[0][1]);
        let (e2x, e2y) = (obb[3][0] - obb[0][0], obb[3][1] - obb[0][1]);
        let e1_len_sq = e1x * e1x + e1y * e1y;
        let e2_len_sq = e2x * e2x + e2y * e2y;
        if e1_len_sq == 0.0 || e2_len_sq == 0.0 {
            return pos;
        }

        // Clamp the object's visual center, then restore the anchor offset.
        let center = [pos[0] - rt.anchor_offset[0], pos[1] - rt.anchor_offset[1]];
        let (ox, oy) = (center[0] - obb[0][0], center[1] - obb[0][1]);
        let u = (ox * e1x + oy * e1y) / e1_len_sq;
        let v = (ox * e2x + oy * e2y) / e2_len_sq;

        // Inset by the object's half-extents (normalized to each edge) so
        // the whole object fits; a boundary smaller than the object pins
        // the center to the middle.
        let inset_u = rt.half_extents[0] / e1_len_sq.sqrt();
        let inset_v = rt.half_extents[1] / e2_len_sq.sqrt();
        let u = if inset_u >= 0.5 {
            0.5
        } else {
            u.clamp(inset_u, 1.0 - inset_u)
        };
        let v = if inset_v >= 0.5 {
            0.5
        } else {
            v.clamp(inset_v, 1.0 - inset_v)
        };

        [
            obb[0][0] + u * e1x + v * e2x + rt.anchor_offset[0],
            obb[0][1] + u * e1y + v * e2y + rt.anchor_offset[1],
        ]
    }

    fn dnd_write_slot(&mut self, slot_id: &str, pos: [f32; 2]) {
        let _ = self
            .player
            .set_position_slot(slot_id, PositionSlot::static_value(pos));
    }

    /// Whether a state-bound gesture is currently allowed to operate.
    /// Unbound gestures (no stateName) are active in every state.
    fn dnd_state_active(&self, rt: &DndRuntime) -> bool {
        match &rt.state_name {
            Some(state_name) => self
                .current_state
                .as_ref()
                .is_some_and(|s| s.name().as_str() == state_name),
            None => true,
        }
    }

    /// Cancel an in-flight drag back to its rest position (no zone actions,
    /// no lock) — used when the gesture's owning state is exited mid-drag.
    fn dnd_cancel_to_rest(&mut self, rt: &mut DndRuntime) {
        if rt.ghost_active {
            // The original never moved: just discard the ghost.
            self.player.renderer.ghost_end(&rt.layer_name);
            rt.ghost_active = false;
            rt.ghost_land = None;
            rt.phase = DndPhase::Idle;
            return;
        }
        let Some(rest) = rt.rest else {
            rt.phase = DndPhase::Idle;
            return;
        };
        let from = self.dnd_slot_position(&rt.slot_id).unwrap_or(rest);

        match rt.tween {
            Some((duration, easing)) if duration > 0.0 => {
                rt.phase = DndPhase::Snapping {
                    from,
                    to: rest,
                    elapsed: 0.0,
                    duration,
                    easing,
                    zone_index: None,
                };
            }
            _ => self.dnd_finalize(rt, rest, None),
        }
    }

    /// Route pointer events into the DragAndDrop gesture runtimes.
    fn manage_drag_and_drop(&mut self, event: &Event, x: f32, y: f32) {
        if self.dnd_runtimes.is_empty() {
            return;
        }

        let mut runtimes = std::mem::take(&mut self.dnd_runtimes);

        match event {
            Event::PointerDown { .. } => {
                for rt in &mut runtimes {
                    self.dnd_try_grab(rt, x, y);
                }
            }
            Event::PointerMove { .. } => {
                for rt in &mut runtimes {
                    if let DndPhase::Held { offset } = rt.phase {
                        if !self.dnd_state_active(rt) {
                            self.dnd_cancel_to_rest(rt);
                            continue;
                        }
                        if rt.ghost_active {
                            // The ghost rides the pointer in canvas pixels;
                            // the original (and its slot) stays parked.
                            self.player.renderer.ghost_offset(
                                &rt.layer_name,
                                x - rt.ghost_origin[0],
                                y - rt.ghost_origin[1],
                            );
                        } else {
                            let slot_id = rt.slot_id.clone();
                            let [cx, cy] = self.to_comp(x, y);
                            let pos =
                                self.dnd_clamp_to_boundary(rt, [cx + offset[0], cy + offset[1]]);
                            self.dnd_write_slot(&slot_id, pos);
                        }
                        let on_drag = rt.on_drag.clone();
                        for action in on_drag {
                            let _ = action.execute(self, true, false);
                        }
                    }
                }
            }
            Event::PointerUp { .. } => {
                for rt in &mut runtimes {
                    if matches!(rt.phase, DndPhase::Held { .. }) {
                        // Releasing after the owning state exited is a
                        // cancel, not a drop: no zones, no actions.
                        if !self.dnd_state_active(rt) {
                            self.dnd_cancel_to_rest(rt);
                        } else {
                            self.dnd_resolve_drop(rt, x, y);
                        }
                    }
                }
            }
            _ => {}
        }

        self.dnd_runtimes = runtimes;
    }

    fn dnd_try_grab(&mut self, rt: &mut DndRuntime, x: f32, y: f32) {
        if rt.locked || matches!(rt.phase, DndPhase::Held { .. }) {
            return;
        }
        if !self.dnd_state_active(rt) {
            return;
        }

        // Pickup uses the SHAPE-accurate hit test: a star is not grabbed
        // by the empty corners of its bounding box.
        let hit = self
            .player
            .renderer
            .hit_test_precise(Point { x, y }, &rt.layer_name)
            .unwrap_or(false);
        if !hit {
            return;
        }

        let Some(current) = self.dnd_slot_position(&rt.slot_id) else {
            return;
        };
        // All gesture math runs in comp units; the pointer arrives in
        // canvas pixels and is converted exactly once.
        let pointer = self.to_comp(x, y);

        // First grab captures the rest position (where a miss returns to),
        // the anchor offset (transform position minus rendered center,
        // used by the boundary clamp to keep the VISUAL rect inside), and
        // the half extents. First grab only: the object is guaranteed
        // settled in the rendered scene then, whereas bounds lag slot
        // writes by one canvas update on later grabs.
        if rt.rest.is_none() {
            rt.rest = Some(current);
            rt.anchor_offset = self
                .layer_center(&rt.layer_name)
                .map(|c| [current[0] - c[0], current[1] - c[1]])
                .unwrap_or([0.0, 0.0]);
            rt.half_extents = self
                .layer_bounds_comp(&rt.layer_name)
                .map(|c| {
                    [
                        ((c[1][0] - c[0][0]).hypot(c[1][1] - c[0][1])) / 2.0,
                        ((c[3][0] - c[0][0]).hypot(c[3][1] - c[0][1])) / 2.0,
                    ]
                })
                .unwrap_or([0.0, 0.0]);
        }

        // A ghost mid-glide finishes before it can be grabbed again.
        if rt.ghost_active && matches!(rt.phase, DndPhase::Snapping { .. }) {
            return;
        }

        // Grabbing mid-snap cancels the tween, and grabbing a tracked
        // object un-docks it; either way the object continues from
        // wherever it currently is.
        rt.tracking = None;

        // Ghost mode: park the original, drag a frozen duplicate. The
        // slot is untouched until the ghost lands. Falls back to a normal
        // slot drag if the duplicate cannot be created.
        if rt.ghost {
            rt.ghost_active = self.player.renderer.ghost_begin(&rt.layer_name);
            if rt.ghost_active {
                rt.ghost_origin = [x, y];
                rt.phase = DndPhase::Held { offset: [0.0, 0.0] };
                let on_grab = rt.on_grab.clone();
                for action in on_grab {
                    let _ = action.execute(self, true, false);
                }
                return;
            }
        }

        // The pointer-to-object offset is always preserved, so the object
        // never jumps to center itself on the pointer at pickup.
        let offset = [current[0] - pointer[0], current[1] - pointer[1]];
        rt.phase = DndPhase::Held { offset };

        // onGrab hooks: let the state machine react to the gesture starting
        // (e.g. Fire an event that transitions into a "dragging" state).
        let on_grab = rt.on_grab.clone();
        for action in on_grab {
            let _ = action.execute(self, true, false);
        }
    }

    fn dnd_resolve_drop(&mut self, rt: &mut DndRuntime, x: f32, y: f32) {
        // onDrop hooks run first, regardless of the drop outcome — this is
        // where "released" events belong (threshold-style release logic
        // lives in transition guards, not zones).
        let on_drop = rt.on_drop.clone();
        for action in on_drop {
            let _ = action.execute(self, true, false);
        }

        // No drop zones = lifecycle-only gesture: no snap, no return. The
        // object stays wherever the release (or a live binding) leaves it.
        // With a ghost, "wherever" is the release point: the slot is
        // written once and the ghost retires.
        if rt.zones.is_empty() {
            if rt.ghost_active {
                let pos = self.to_comp(x, y);
                let slot_id = rt.slot_id.clone();
                self.dnd_write_slot(&slot_id, pos);
                self.player.renderer.ghost_end(&rt.layer_name);
                rt.ghost_active = false;
                rt.rest = Some(pos);
            } else {
                rt.rest = self.dnd_slot_position(&rt.slot_id).or(rt.rest);
            }
            rt.phase = DndPhase::Idle;
            return;
        }

        let zone_hit = rt.zones.iter().position(|zone| {
            self.player
                .renderer
                .hit_test(Point { x, y }, &zone.layer_name)
                .unwrap_or(false)
        });

        let miss_pos = || rt.rest.unwrap_or_else(|| self.to_comp(x, y));
        let (target, zone_index) = match zone_hit {
            Some(zi) => {
                // Snap target: explicit override, else the zone layer's
                // CURRENT transform position (matrix translation — exact
                // authored semantics, animated/parented zones included),
                // with the rendered-bounds center as fallback.
                let target = rt.zones[zi].snap.or_else(|| {
                    self.layer_position(&rt.zones[zi].layer_name)
                        .or_else(|| self.layer_center(&rt.zones[zi].layer_name))
                });
                match target {
                    Some(t) => (t, Some(zi)),
                    // Zone hit but no derivable snap target: treat as miss.
                    None => (miss_pos(), None),
                }
            }
            None => (miss_pos(), None),
        };

        // Track zones dock INSTANTLY: an engine tween cannot chase a moving
        // endpoint, and the per-tick follow takes over from the next tick.
        let tracked = zone_index.is_some_and(|zi| rt.zones[zi].track);

        // Ghost mode. Dock: the ghost retires at the release point and
        // the ORIGINAL glides rest -> dock through the normal slot tween
        // below — the visible "item travels to its destination" beat.
        // Miss: the ghost glides back over the parked original (canvas
        // offset -> zero) and nothing else moves.
        if rt.ghost_active {
            if zone_index.is_some() {
                self.player.renderer.ghost_end(&rt.layer_name);
                rt.ghost_active = false;
                rt.ghost_land = None;
                // Fall through to the slot glide: from = the slot's
                // current (rest) position, to = the dock target.
            } else {
                let from = [x - rt.ghost_origin[0], y - rt.ghost_origin[1]];
                match rt.tween {
                    Some((duration, easing)) if duration > 0.0 => {
                        rt.ghost_land = Some(target);
                        rt.phase = DndPhase::Snapping {
                            from,
                            to: [0.0, 0.0],
                            elapsed: 0.0,
                            duration,
                            easing,
                            zone_index: None,
                        };
                    }
                    _ => {
                        self.player.renderer.ghost_end(&rt.layer_name);
                        rt.ghost_active = false;
                        self.dnd_finalize(rt, target, None);
                    }
                }
                return;
            }
        }

        let from = self.dnd_slot_position(&rt.slot_id).unwrap_or(target);

        match rt.tween {
            Some((duration, easing)) if duration > 0.0 && !tracked => {
                rt.phase = DndPhase::Snapping {
                    from,
                    to: target,
                    elapsed: 0.0,
                    duration,
                    easing,
                    zone_index,
                };
            }
            _ => self.dnd_finalize(rt, target, zone_index),
        }
    }

    /// Land the object: write the exact target, update rest, run the drop
    /// zone's actions (if docking), and apply its lock.
    fn dnd_finalize(&mut self, rt: &mut DndRuntime, target: [f32; 2], zone_index: Option<usize>) {
        let slot_id = rt.slot_id.clone();
        self.dnd_write_slot(&slot_id, target);
        rt.rest = Some(target);
        rt.phase = DndPhase::Idle;

        if let Some(zi) = zone_index {
            rt.locked = rt.locked || rt.zones[zi].lock;
            if rt.zones[zi].track {
                rt.tracking = Some(zi);
            }
            let actions = rt.zones[zi].actions.clone();
            for action in actions {
                let _ = action.execute(self, true, false);
            }
        }
    }

    /// Advance in-flight snap tweens (non-blocking; runs every tick).
    fn advance_drag_and_drop(&mut self, dt: f32) {
        if self.dnd_runtimes.is_empty() {
            return;
        }

        let mut runtimes = std::mem::take(&mut self.dnd_runtimes);
        for rt in &mut runtimes {
            // Lazy state-scope enforcement: a transition away from the
            // owning state (from any cause — guards, tweens, actions)
            // cancels an in-flight drag within a tick.
            if matches!(rt.phase, DndPhase::Held { .. }) && !self.dnd_state_active(rt) {
                self.dnd_cancel_to_rest(rt);
                continue;
            }

            // Tracking dock: follow the zone by reading its rendered
            // center each tick and rewriting the slot. A static zone costs
            // nothing (unchanged target skips the write); a moving zone is
            // followed one canvas update behind, with no expression engine
            // involved.
            if let (DndPhase::Idle, Some(zi)) = (rt.phase, rt.tracking) {
                if self.dnd_state_active(rt) {
                    let zone_name = &rt.zones[zi].layer_name;
                    if let Some(target) = self
                        .layer_position(zone_name)
                        .or_else(|| self.layer_center(zone_name))
                    {
                        if rt.rest != Some(target) {
                            let slot_id = rt.slot_id.clone();
                            self.dnd_write_slot(&slot_id, target);
                            rt.rest = Some(target);
                        }
                    }
                }
                continue;
            }

            let DndPhase::Snapping {
                from,
                to,
                elapsed,
                duration,
                easing,
                zone_index,
            } = rt.phase
            else {
                continue;
            };

            let elapsed = elapsed + dt;
            if elapsed >= duration {
                if rt.ghost_active {
                    // The ghost lands: retire it and write the slot once.
                    self.player.renderer.ghost_end(&rt.layer_name);
                    rt.ghost_active = false;
                    let land = rt
                        .ghost_land
                        .take()
                        .or(rt.rest)
                        .unwrap_or(to);
                    self.dnd_finalize(rt, land, zone_index);
                } else {
                    self.dnd_finalize(rt, to, zone_index);
                }
            } else {
                let progress = crate::tween::TweenState::eased_progress(elapsed / duration, easing);
                let pos = drag_and_drop::lerp2(from, to, progress);
                if rt.ghost_active {
                    // from/to are canvas-pixel OFFSETS in ghost mode.
                    self.player
                        .renderer
                        .ghost_offset(&rt.layer_name, pos[0], pos[1]);
                } else {
                    let slot_id = rt.slot_id.clone();
                    self.dnd_write_slot(&slot_id, pos);
                }
                rt.phase = DndPhase::Snapping {
                    from,
                    to,
                    elapsed,
                    duration,
                    easing,
                    zone_index,
                };
            }
        }
        self.dnd_runtimes = runtimes;
    }

    // ── PathDrag interaction runtime (prototype) ──────────────────────

    fn path_drag_state_active(&self, rt: &PathDragRuntime) -> bool {
        match &rt.state_name {
            Some(state_name) => self
                .current_state
                .as_ref()
                .is_some_and(|s| s.name().as_str() == state_name),
            None => true,
        }
    }

    /// Project the pointer onto the path (windowed, branch-local) and
    /// publish progress as an input write (run_pipeline = true, so
    /// bindings and guards react). Returns the published progress.
    fn path_drag_publish(&mut self, rt: &mut PathDragRuntime, x: f32, y: f32) -> Option<f32> {
        let (_point, progress) = rt.project_windowed([x, y])?;
        let name = rt.progress_input.clone();
        let _ = self.set_numeric_input(&name, progress, true, false);
        Some(progress)
    }

    /// Half of a layer's larger rendered dimension in comp units — the
    /// "radius" used for arc-proximity capture.
    fn layer_half_extent(&self, layer_name: &str) -> f32 {
        self.layer_bounds_comp(layer_name)
            .map(|c| {
                let w = (c[1][0] - c[0][0]).hypot(c[1][1] - c[0][1]);
                let h = (c[3][0] - c[0][0]).hypot(c[3][1] - c[0][1]);
                w.max(h) / 2.0
            })
            .unwrap_or(0.0)
    }

    /// Advance in-flight dock glides: eased progress from the release
    /// point to the zone's on-path position, published as ordinary input
    /// writes with onDrag running each tick — the same pipeline as a live
    /// drag, so whatever converts progress to visuals keeps working.
    fn advance_path_drag(&mut self, dt: f32) {
        if self.pathdrag_runtimes.is_empty() {
            return;
        }

        let mut runtimes = std::mem::take(&mut self.pathdrag_runtimes);

        for rt in &mut runtimes {
            let Some(mut snap) = rt.snapping else {
                continue;
            };
            // A scoped gesture whose owning state exited drops the glide,
            // mirroring the drag-cancel semantics.
            if !self.path_drag_state_active(rt) {
                rt.snapping = None;
                continue;
            }

            snap.elapsed += dt;
            let done = snap.elapsed >= snap.duration;
            let t = if done {
                snap.to
            } else {
                let p = crate::tween::TweenState::eased_progress(
                    snap.elapsed / snap.duration,
                    snap.easing,
                );
                snap.from + (snap.to - snap.from) * p
            };
            rt.snapping = if done { None } else { Some(snap) };

            let name = rt.progress_input.clone();
            let _ = self.set_numeric_input(&name, t, true, false);
            let on_drag = rt.on_drag.clone();
            for action in on_drag {
                let _ = action.execute(self, true, false);
            }
        }

        self.pathdrag_runtimes = runtimes;
    }

    fn manage_path_drag(&mut self, event: &Event, x: f32, y: f32) {
        if self.pathdrag_runtimes.is_empty() {
            return;
        }

        let mut runtimes = std::mem::take(&mut self.pathdrag_runtimes);

        match event {
            Event::PointerDown { .. } => {
                for rt in &mut runtimes {
                    if rt.held || rt.locked || !self.path_drag_state_active(rt) {
                        continue;
                    }
                    // Pickup uses the SHAPE-accurate hit test (raw canvas
                    // pixels; projection math below runs in comp units).
                    let hit = self
                        .player
                        .renderer
                        .hit_test_precise(Point { x, y }, &rt.layer_name)
                        .unwrap_or(false);
                    if !hit || rt.samples.is_empty() {
                        continue;
                    }
                    rt.held = true;
                    // Grabbing cancels an in-flight dock glide and un-docks.
                    // Re-seed branch locality at the grab point, then
                    // publish BEFORE onGrab so a state entered by the hook
                    // binds against fresh values.
                    rt.snapping = None;
                    let pointer = self.to_comp(x, y);
                    rt.seed(pointer);
                    self.path_drag_publish(rt, pointer[0], pointer[1]);
                    let on_grab = rt.on_grab.clone();
                    for action in on_grab {
                        let _ = action.execute(self, true, false);
                    }
                }
            }
            Event::PointerMove { .. } => {
                for rt in &mut runtimes {
                    if !rt.held {
                        continue;
                    }
                    if !self.path_drag_state_active(rt) {
                        rt.held = false; // cancelled by state exit: no hooks
                        continue;
                    }
                    // onDrag runs after the publish, so its actions read
                    // the fresh progress.
                    let pointer = self.to_comp(x, y);
                    self.path_drag_publish(rt, pointer[0], pointer[1]);
                    let on_drag = rt.on_drag.clone();
                    for action in on_drag {
                        let _ = action.execute(self, true, false);
                    }
                }
            }
            Event::PointerUp { .. } => {
                for rt in &mut runtimes {
                    if !rt.held {
                        continue;
                    }
                    rt.held = false;
                    if !self.path_drag_state_active(rt) {
                        continue; // cancelled: no final publish, no hooks
                    }
                    let pointer = self.to_comp(x, y);
                    let final_t = self.path_drag_publish(rt, pointer[0], pointer[1]);

                    // Path-mode drop zones are dock points ON the path,
                    // captured by ARC PROXIMITY, not pointer hit-testing:
                    // the pointer may sit far off the path while the object
                    // rides its projection (and dots may be tiny), so what
                    // matters is whether the OBJECT stopped within reach of
                    // a dot. Reach = object half-size + zone half-size
                    // (rendered bounds), measured along the path — which
                    // also means a dot on an adjacent turn of a spiral can
                    // never falsely capture. Nearest qualifying zone wins;
                    // progress snaps to the zone's own on-path position.
                    let mut zone_hit: Option<(usize, f32, f32)> = None; // (zi, zone_t, arc_dist)
                    if let Some(t) = final_t {
                        let zone_ts: Vec<(usize, f32)> = rt
                            .zones
                            .iter()
                            .enumerate()
                            .filter_map(|(zi, zone)| {
                                self.layer_position(&zone.layer_name)
                                    .or_else(|| self.layer_center(&zone.layer_name))
                                    .and_then(|c| rt.progress_at_point(c))
                                    .map(|zone_t| (zi, zone_t))
                            })
                            .collect();

                        let reach_base = self.layer_half_extent(&rt.layer_name);
                        for &(zi, zone_t) in &zone_ts {
                            let arc_dist = (t - zone_t).abs() * rt.total_len;
                            let reach =
                                reach_base + self.layer_half_extent(&rt.zones[zi].layer_name);
                            if arc_dist <= reach
                                && zone_hit.is_none_or(|(_, _, best)| arc_dist < best)
                            {
                                zone_hit = Some((zi, zone_t, arc_dist));
                            }
                        }

                        // Uncaptured release: dockFallback ratchets to a
                        // zone anyway — "previous" to the nearest one at
                        // or behind the release progress, "nearest" to
                        // the closest in either direction. No candidate
                        // (e.g. released before the first dock point)
                        // leaves zone = 0 and the machine decides.
                        if zone_hit.is_none() {
                            let mode = rt.dock_fallback.as_deref();
                            for &(zi, zone_t) in &zone_ts {
                                let arc_dist = match mode {
                                    Some("previous") if zone_t <= t => (t - zone_t) * rt.total_len,
                                    Some("nearest") => (t - zone_t).abs() * rt.total_len,
                                    _ => continue,
                                };
                                if zone_hit.is_none_or(|(_, _, best)| arc_dist < best) {
                                    zone_hit = Some((zi, zone_t, arc_dist));
                                }
                            }
                        }
                    }
                    if let Some((_, zone_t, _)) = zone_hit {
                        match (rt.tween, final_t) {
                            // Glide ALONG THE PATH into the dock:
                            // advance_path_drag animates the progress
                            // input and runs onDrag each tick, so the
                            // object slides rather than teleports.
                            (Some((duration, easing)), Some(from))
                                if duration > 0.0 && from != zone_t =>
                            {
                                rt.snapping = Some(PathSnap {
                                    from,
                                    to: zone_t,
                                    elapsed: 0.0,
                                    duration,
                                    easing,
                                });
                            }
                            _ => {
                                let name = rt.progress_input.clone();
                                let _ = self.set_numeric_input(&name, zone_t, true, false);
                            }
                        }
                    }
                    let zone_hit = zone_hit.map(|(zi, _, _)| zi);

                    // onDrop hooks run after the progress publish (so a
                    // Fired event's guards see the snapped value), then
                    // the zone's own actions, mirroring free mode.
                    let on_drop = rt.on_drop.clone();
                    for action in on_drop {
                        let _ = action.execute(self, true, false);
                    }
                    if let Some(zi) = zone_hit {
                        rt.locked = rt.locked || rt.zones[zi].lock;
                        let actions = rt.zones[zi].actions.clone();
                        for action in actions {
                            let _ = action.execute(self, true, false);
                        }
                    }
                }
            }
            _ => {}
        }

        self.pathdrag_runtimes = runtimes;
    }

    // Set the current state to the target state
    // Manage performing entry and exit actions
    // As well as executing the state's type (Currently on PlaybackState has an effect on playback)
    fn set_current_state(
        &mut self,
        state_name: &str,
        causing_transition: Option<&Transition>,
        called_from_global: bool,
    ) -> Result<(), StateMachineEngineError> {
        let new_state = self.get_state(state_name);
        // We have a new state
        if let Some(new_state) = new_state {
            // Emit transtion occured event
            self.observe_on_transition(&self.get_current_state_name(), new_state.name());
            // Perform exit actions on the current state if there is one.
            if self.current_state.is_some() {
                let state = self.current_state.take();
                // Now use the extracted information
                if let Some(state) = state {
                    if !called_from_global {
                        let _ = state.exit(self);
                    }
                    // Don't forget to put things back
                    // new_state becomes the current state
                    self.current_state = Some(state);
                }
            }
            // Emit transtion occured event
            self.observe_on_state_exit(&self.get_current_state_name());

            // Since the blended transition will take time
            // We have to save the target state and do the final transition when tweening has completed
            // The state machine is alerted of tweening finishing because the player calls the resume_from_tweening() method
            if let Some(causing_transition) = causing_transition {
                // If we dealing with a tweened transition
                if let Transition::Tweened { .. } = causing_transition {
                    let segment_ref = match &new_state {
                        State::PlaybackState { segment, .. } => segment.as_deref(),
                        _ => None,
                    };
                    let is_reverse = match &new_state {
                        State::PlaybackState { mode, .. } => {
                            matches!(mode, Some(Mode::Reverse | Mode::ReverseBounce))
                        }
                        _ => false,
                    };
                    match &new_state {
                        State::PlaybackState {
                            animation: target_animation,
                            ..
                        } => {
                            let same_animation = self
                                .current_state
                                .as_ref()
                                .map(|s| s.animation() == target_animation.as_str())
                                .unwrap_or(false);

                            if same_animation {
                                let target_frame = if let Some(target_segment) = segment_ref {
                                    let marker_lookup = self
                                        .player
                                        .markers()
                                        .iter()
                                        .find(|m| m.name.to_str() == Ok(target_segment));

                                    marker_lookup.map(|m| {
                                        if is_reverse {
                                            m.segment.end.min(self.player.total_frames() - 1.0)
                                        } else {
                                            m.segment.start
                                        }
                                    })
                                } else {
                                    Some(if is_reverse {
                                        self.player.total_frames() - 1.0
                                    } else {
                                        0.0
                                    })
                                };

                                if let Some(target_frame) = target_frame {
                                    let tween_result = self.player.tween(
                                        target_frame,
                                        causing_transition.duration(),
                                        causing_transition.easing(),
                                    );

                                    if tween_result.is_ok() {
                                        self.prepare_state_slot_tween(&new_state);
                                        self.tween_transition_target_state =
                                            Some(new_state.clone());
                                        self.tween_target_frame = Some(target_frame);
                                        self.status = StateMachineEngineStatus::Tweening;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        State::GlobalState { .. } => {
                            return Ok(());
                        }
                    }
                }
            }

            // Release or drop the previous state's slot overlay before the
            // new state applies. On an animation change the outgoing slot
            // state doesn't survive the load, so bookkeeping is dropped
            // without restore writes.
            let animation_changing = {
                let target_animation = new_state.animation();
                !target_animation.is_empty()
                    && self
                        .player
                        .animation_id()
                        .map(|id| id.to_bytes() != target_animation.as_bytes())
                        .unwrap_or(true)
            };
            if animation_changing {
                self.reset_state_slot_bookkeeping();
            } else {
                self.release_state_slots(Some(&new_state));
            }

            // Assign the new state to the current_state
            self.current_state = Some(new_state);

            // Emit transtion occured event
            self.observe_on_state_entered(&self.get_current_state_name());
            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();
            // Now use the extracted information
            if let Some(state) = state {
                // Enter the state
                let _ = state.enter(self);

                // Apply the state's declared slots (after enter, so a
                // changed animation is already loaded and reseeded).
                let declared: Vec<StateSlot> = state.state_slots().cloned().unwrap_or_default();
                self.apply_state_slots(&declared);

                // Don't forget to put things back
                // new_state becomes the current state
                self.current_state = Some(state);
            } else {
                return Err(StateMachineEngineError::SetStateError);
            }
            return Ok(());
        }
        Err(StateMachineEngineError::CreationError)
    }

    // Returns: The target state and the causing transition
    fn evaluate_transitions(
        &self,
        state_to_evaluate: &State,
        event: Option<&DotString>,
    ) -> Option<(String, Transition)> {
        let transitions = state_to_evaluate.transitions();
        let mut guardless_transition: Option<&Transition> = None;

        for transition in transitions {
            if transition.guards().is_none() || transition.guards().as_ref().unwrap().is_empty() {
                guardless_transition = Some(transition);
            }
            // If in the transitions we need an event, and there wasn't one fired, don't run the checks.
            // If there wasn't an event needed, but we are sending an event, still do the checks.

            // Guards on a transition are evaluated in order of priority, all of them have to be valid to transition (&& not ||).
            else if (transition.transitions_contain_event() && event.is_some())
                || (!transition.transitions_contain_event() && event.is_none())
            {
                if let Some(guards) = transition.guards() {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            transitions::guard::Guard::Numeric { .. } => {
                                if !guard
                                    .numeric_input_is_satisfied(&self.inputs, self.elapsed_time)
                                {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&self.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&self.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if event.is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = event {
                                    if !guard.event_input_is_satisfied(event.as_str()) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    /* If all guard are satsified, take the transition as they are in order of priority inside the vec */
                    if all_guards_satisfied {
                        let target_state = transition.target_state();

                        return Some((target_state.to_string(), transition.clone()));
                    }
                }
            }
        }

        // Enforces the rule that a guardless transition should be taken in to account last
        let target_state = guardless_transition?.target_state();
        Some((target_state.to_string(), guardless_transition?.clone()))
    }

    fn evaluate_global_state(&mut self) -> bool {
        if let Some(state_to_evaluate) = &self.global_state {
            if let Some((target_state, causing_transition)) =
                self.evaluate_transitions(state_to_evaluate, self.curr_event.as_ref())
            {
                self.curr_event = None;

                // Prevent re-entering the current state again
                if target_state == self.get_current_state_name() {
                    return false;
                }

                let success =
                    self.set_current_state(&target_state, Some(&causing_transition), true);

                match success {
                    Ok(()) => {
                        return true;
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
        }
        false
    }

    pub fn run_current_state_pipeline(&mut self) -> Result<(), StateMachineEngineError> {
        // Reset cycle count for each pipeline run
        self.current_cycle_count = 0;

        // If the state machine is tweening, don't run the pipeline
        if self.status == StateMachineEngineStatus::Tweening {
            return Ok(());
        }

        // If the state machine is not running, or there is no current state, return an error
        // Otherwise this will block the pipeline in a loop
        if self.status != StateMachineEngineStatus::Running
            || (self.current_state.is_none() && self.global_state.is_none())
        {
            return Err(StateMachineEngineError::NotRunningError);
        }

        let mut tick = true;

        let mut ignore_global = false;

        while tick {
            // Safety fallback to prevent infinite loops
            tick = false;
            let mut ignore_child = false;

            // --------------- Start infinite loop detection
            if self.detect_cycle() {
                self.current_cycle_count += 1;

                if self.current_cycle_count >= self.max_cycle_count {
                    self.stop();
                    self.observe_on_error("InfiniteLoop");
                    return Err(StateMachineEngineError::InfiniteLoopError);
                }

                // Clear the history to allow for detecting new cycles
                self.state_history.clear();
            }

            if let Some(state) = &self.current_state {
                let name = self.str_interner.intern(state.name());
                self.state_history.push(name);
            }

            // --------------- End infinite loop detection

            // Check if there is a global state
            // If there is, evaluate the transitions of the global state first
            if !ignore_global {
                // Global state returned true meaning it changed the current state
                if self.evaluate_global_state() {
                    // Check the current state, if its tweening, stop immediately
                    if self.status == StateMachineEngineStatus::Tweening {
                        break;
                    }
                    // Therfor we need to re-evaluate the global state.
                    // When we entered the state from global, it made on_entry changes.
                    if self.action_mutated_inputs {
                        ignore_global = false;
                        ignore_child = true;

                        tick = true;
                        self.action_mutated_inputs = false;
                    }
                    if self.curr_event.is_some() {
                        ignore_global = false;
                        ignore_child = true;

                        tick = true;
                    }
                }
            }

            if !ignore_child {
                if let Some(current_state_to_evaluate) = &self.current_state {
                    if let Some((target_state, causing_transition)) = self
                        .evaluate_transitions(current_state_to_evaluate, self.curr_event.as_ref())
                    {
                        self.curr_event = None;

                        let success =
                            self.set_current_state(&target_state, Some(&causing_transition), false);

                        match success {
                            Ok(()) => {
                                // Check the current state, if its tweening, stop immediately
                                if self.status == StateMachineEngineStatus::Tweening {
                                    break;
                                }
                                // Re-evaluate global state, a input was changed
                                if self.action_mutated_inputs {
                                    tick = true;

                                    ignore_global = false;
                                    self.action_mutated_inputs = false;
                                }
                                // Re-evaluate global state, an event was fired
                                else if self.curr_event.is_some() {
                                    tick = true;

                                    ignore_global = false;
                                }
                                // Re-evaluate current state, ignore global since no inputs were changed or events fired
                                else {
                                    tick = true;

                                    ignore_global = true;
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.curr_event = None;

        Ok(())
    }

    fn detect_cycle(&self) -> bool {
        match self.state_history.split_last() {
            Some((last, rest)) => rest.contains(last),
            None => false,
        }
    }

    fn manage_explicit_events(&mut self, event: &Event, x: f32, y: f32) {
        let mut actions_to_execute: Vec<Action> = Vec::new();
        let mut entered_layer = self.pointer_management.curr_entered_layer.clone();

        for interaction in self.interactions(None) {
            if interaction.type_name() == event.type_name() {
                // User defined a specific layer to check if hit
                if let Some(layer) = interaction.get_layer_name() {
                    // If we have a pointer exit event, check if the pointer is outside of the layer
                    if let Event::PointerExit { x, y } = event {
                        if self.pointer_management.curr_entered_layer == *layer
                            && !self
                                .player
                                .renderer
                                .hit_test(Point { x: *x, y: *y }, layer)
                                .unwrap_or(false)
                        {
                            entered_layer = DotString::empty();
                            actions_to_execute.extend(interaction.get_actions().clone());
                        }
                    } else {
                        // Hit check will return true if the layer was hit
                        if self
                            .player
                            .renderer
                            .hit_test(Point { x, y }, layer)
                            .unwrap_or(false)
                        {
                            entered_layer = layer.clone();
                            actions_to_execute.extend(interaction.get_actions().clone());
                        }
                    }
                } else {
                    // No layer was specified, add all actions
                    actions_to_execute.extend(interaction.get_actions().clone());
                }
            }
        }

        self.pointer_management.curr_entered_layer = entered_layer;

        for action in actions_to_execute {
            // Run the pipeline because interactions are outside of the evaluation pipeline loop
            let _ = action.execute(self, true, false);
        }
    }

    fn manage_cross_platform_events(&mut self, event: &Event, x: f32, y: f32) {
        let mut actions_to_execute = Vec::new();

        // Manage pointerMove interactions
        if event.type_name() == "PointerMove" {
            let pointer_move_interactions = self.interactions(Some(event_type_name!(PointerMove)));

            for interaction in pointer_move_interactions {
                if let Interaction::PointerMove {
                    state_name,
                    actions,
                } = interaction
                {
                    // State-scoped move handlers (OnComplete-style) are
                    // inert outside their owning state.
                    if let Some(state_name) = state_name {
                        let active = self
                            .current_state
                            .as_ref()
                            .is_some_and(|s| s.name() == state_name);
                        if !active {
                            continue;
                        }
                    }
                    actions_to_execute.extend(actions.clone());
                }
            }
        }

        // Check if we've moved the pointer over any of the pointerEnter/Exit interactions
        // If we've changed layers, perform exit actions
        // If we don't hit any layers, perform exit actions
        let mut hit = false;
        let old_layer = self.pointer_management.curr_entered_layer.clone();

        // Loop through all layers we're listening to
        for i in 0..self.pointer_management.listened_layers.len() {
            // We're only interested in the listened layers that need enter / exit event
            if (self.pointer_management.listened_layers[i].1 == event_type_name!(PointerEnter)
                || self.pointer_management.listened_layers[i].1 == event_type_name!(PointerExit))
                && self
                    .player
                    .renderer
                    .hit_test(
                        Point { x, y },
                        &self.pointer_management.listened_layers[i].0,
                    )
                    .unwrap_or(false)
            {
                hit = true;

                // If it's that same current layer, do nothing
                if self.pointer_management.curr_entered_layer
                    == self.pointer_management.listened_layers[i].0
                {
                    break;
                }

                self.pointer_management.curr_entered_layer =
                    self.pointer_management.listened_layers[i].0.clone();

                // Get all pointer_enter interactions
                // Add their actions if their layer name matches the current layer name in loop
                for interaction in self.interactions(Some(event_type_name!(PointerEnter))) {
                    if let Some(interaction_layer_name) = interaction.get_layer_name() {
                        if *interaction_layer_name == self.pointer_management.curr_entered_layer {
                            actions_to_execute.extend(interaction.get_actions().clone());
                        }
                    }
                }
            }
        }

        // We didn't hit any listened layers
        if !hit && !old_layer.is_empty() {
            self.pointer_management.curr_entered_layer = DotString::empty();

            let pointer_exit_interactions = self.interactions(Some(event_type_name!(PointerExit)));

            // Add the actions of every PointerExit interaction that depended on the layer we've just exited
            for interaction in pointer_exit_interactions {
                if let Some(interaction_layer_name) = interaction.get_layer_name() {
                    // We've exited the desired layer, add its actions to execute
                    if *interaction_layer_name == old_layer {
                        actions_to_execute.extend(interaction.get_actions().clone());
                    }
                }
            }
        }

        for action in actions_to_execute {
            // Run the pipeline because interactions are outside of the evaluation pipeline loop
            let _ = action.execute(self, true, false);
        }
    }

    // How pointer event are managed depending on the interaction's event and the sent event.
    // Since we can't detect PointerMove on mobile, we can still check PointerDown/Up and see if it's entered or exited a layer.
    //
    // | -------------------------------- | ----------------------------- | ----------- |
    // | Interaction Event type              | Web                           | Mobile      |
    // | -------------------------------- | ----------------------------- | ----------- |
    // | PointerDown (No Layer)           | PointerDown                   | PointerDown |
    // | PointerDown (With Layer)         | PointerDown                   | PointerDown |
    // | PointerUp (No Layer)             | PointerUp                     | PointerUp   |
    // | PointerUp (With Layer)           | PointerUp                     | PointerUp   |
    // | PointerMove (No Layer)           | PointerMove                   | PointerDown |
    // | PointerEnter (No Layer)          | PointerEnter                  | Not avail.  |
    // | PointerEnter (With Layer)        | PointerMove + PointerEnter    | PointerDown |
    // | PointerExit (No Layer)           | PointerExit                   | Not avail.  |
    // | PointerExit (With Layer)         | PointerMove + PointerExit     | PointerUp   |
    // | Click (With Layer)               | Click                         | Tap         |
    // | Click (No Layer)                 | Click                         | Tap         |
    // | ---------------------------------|-------------------------------| ----------- |

    // Notes:
    // Atm, PointerEnter/Exit without layers is not supported on mobile.
    // This is because if we allow pointerDown to activate PointerEnter/Exit,
    // It would override PointerDown with layers, which is not a great experience.
    // With the current setup we can have an action that happens when the cursor is over the canvas
    // and another action that happens when the cursor is over a specific layer.
    fn manage_pointer_event(&mut self, event: &Event, x: f32, y: f32) {
        self.pointer_management.pointer_x = x;
        self.pointer_management.pointer_y = y;

        // Gesture runtimes see every pointer event first.
        self.manage_drag_and_drop(event, x, y);
        self.manage_path_drag(event, x, y);

        // This will handle PointerDown, PointerUp, PointerEnter, PointerExit, Click
        if event.type_name() != "PointerMove" {
            self.manage_explicit_events(event, x, y);
        }

        // We're left with PointerMove
        // Also perform checks for PointerDown and PointerUp, a mobile framework could of sent them and validate PointerEnter/Exit interactions.
        if event.type_name() == "PointerMove"
            || event.type_name() == "PointerDown"
            || event.type_name() == "PointerUp"
        {
            self.manage_cross_platform_events(event, x, y);
        }
    }

    fn manage_player_events(&mut self, event: &Event) {
        let mut actions_to_execute = Vec::new();

        for interaction in self.interactions(Some(event.type_name())) {
            if let Interaction::OnComplete {
                state_name,
                actions,
            } = interaction
            {
                if let Some(current_state) = &self.current_state {
                    if *current_state.name() == *state_name {
                        actions_to_execute.extend(actions.clone());
                    }
                }
            }
            if let Interaction::OnLoopComplete {
                state_name,
                actions,
            } = interaction
            {
                if let Some(current_state) = &self.current_state {
                    if *current_state.name() == *state_name {
                        actions_to_execute.extend(actions.clone());
                    }
                }
            }
        }

        for action in actions_to_execute {
            // Run the pipeline because interactions are outside of the evaluation pipeline loop
            let _ = action.execute(self, true, false);
        }
    }

    pub fn post_event(&mut self, event: &Event) {
        self.pointer_management.most_recent_event = Some(event.clone());

        if event.type_name().contains("Pointer") || event.type_name().contains("Click") {
            self.manage_pointer_event(event, event.x(), event.y());
        } else {
            self.manage_player_events(event);
        }
    }

    /**
     * Force a state change to the target state. Will not input an evaluation
     * after entering the target state.
     *
     * @params state_name: The name of the state to change to.
     * @params do_tick: If true, the state machine will run the transition evaluation pipeline after changing the state.
     */
    pub fn override_current_state(
        &mut self,
        state_name: &str,
        do_tick: bool,
    ) -> Result<(), crate::PlayerError> {
        if self.set_current_state(state_name, None, false).is_err() {
            return Err(crate::PlayerError::Unknown);
        }

        if do_tick {
            return if self.run_current_state_pipeline().is_ok() {
                Ok(())
            } else {
                Err(crate::PlayerError::Unknown)
            };
        }

        Ok(())
    }

    pub fn get_current_state_name(&self) -> String {
        if let Some(state) = &self.current_state {
            return state.name().as_str().to_owned();
        }

        "".to_string()
    }

    fn observe_on_state_entered(&mut self, entering_state: &str) {
        let state = self.str_interner.intern(entering_state);
        self.event_queue
            .push(StateMachineEvent::StateEntered { state });
    }

    fn observe_on_state_exit(&mut self, leaving_state: &str) {
        let state = self.str_interner.intern(leaving_state);
        self.event_queue
            .push(StateMachineEvent::StateExit { state });
    }

    fn observe_on_transition(&mut self, previous_state: &str, new_state: &str) {
        let previous_state = self.str_interner.intern(previous_state);
        let new_state = self.str_interner.intern(new_state);
        self.event_queue.push(StateMachineEvent::Transition {
            previous_state,
            new_state,
        });
    }

    pub fn observe_internal_event(&mut self, message: &str) {
        let message = self.str_interner.intern(message);
        self.internal_event_queue
            .push(StateMachineInternalEvent::Message { message });
    }

    pub fn observe_custom_event(&mut self, message: &str) {
        let message = self.str_interner.intern(message);
        self.event_queue
            .push(StateMachineEvent::CustomEvent { message });
    }

    pub fn observe_on_error(&mut self, message: &str) {
        let message = self.str_interner.intern(message);
        self.event_queue.push(StateMachineEvent::Error { message });
    }

    pub fn observe_string_input_value_change(
        &mut self,
        input_name: &str,
        old_value: &str,
        new_value: &str,
    ) {
        if old_value == new_value {
            return;
        }
        let name = self.str_interner.intern(input_name);
        let old_value = self.str_interner.intern(old_value);
        let new_value = self.str_interner.intern(new_value);
        self.event_queue.push(StateMachineEvent::StringInputChange {
            name,
            old_value,
            new_value,
        });
    }

    pub fn observe_numeric_input_value_change(
        &mut self,
        input_name: &str,
        old_value: f32,
        new_value: f32,
    ) {
        if old_value == new_value {
            return;
        }
        let name = self.str_interner.intern(input_name);
        self.event_queue
            .push(StateMachineEvent::NumericInputChange {
                name,
                old_value,
                new_value,
            });
    }

    pub fn observe_boolean_input_value_change(
        &mut self,
        input_name: &str,
        old_value: bool,
        new_value: bool,
    ) {
        if old_value == new_value {
            return;
        }
        let name = self.str_interner.intern(input_name);
        self.event_queue
            .push(StateMachineEvent::BooleanInputChange {
                name,
                old_value,
                new_value,
            });
    }

    pub fn observe_on_start(&mut self) {
        self.event_queue.push(StateMachineEvent::Start);
    }

    pub fn observe_on_stop(&mut self) {
        self.event_queue.push(StateMachineEvent::Stop);
    }

    pub fn observe_on_input_fired(&mut self, input_name: &str) {
        let name = self.str_interner.intern(input_name);
        self.event_queue
            .push(StateMachineEvent::InputFired { name });
    }

    fn check_completion(&mut self) {
        match self.player.pop_completion_event() {
            CompletionEvent::Completed => {
                self.post_event(&Event::OnComplete);
            }
            CompletionEvent::LoopCompleted => {
                self.post_event(&Event::OnLoopComplete);
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, dt: f32) -> Result<bool, crate::PlayerError> {
        // Advance DragAndDrop snap tweens BEFORE the player renders, so this
        // tick's slot writes are flushed in this tick's render (hit-testing
        // depends on the rendered scene being current).
        if self.status != StateMachineEngineStatus::Stopped {
            self.advance_drag_and_drop(dt);
            self.advance_path_drag(dt);
        }

        let ticked = self.player.tick(dt);

        self.check_completion();

        // Advance slot interpolations with the tween's eased progress. When
        // the tween just completed, progress is gone — resume_from_tweening
        // below snaps the exact final values.
        if self.status == StateMachineEngineStatus::Tweening {
            if let Some(progress) = self.player.tween_progress() {
                self.apply_slot_lerps(progress);
            }
        }

        let needs_resume =
            self.status == StateMachineEngineStatus::Tweening && !self.player.is_tweening();

        if needs_resume {
            self.resume_from_tweening();
        }

        if self.status != StateMachineEngineStatus::Stopped {
            self.elapsed_time_increment(dt);

            // Re-evaluate the pipeline if either the GlobalState routes by
            // elapsedTime (every tick has to check it) or the current
            // PlaybackState has its own elapsedTime guard.
            if self.status == StateMachineEngineStatus::Running {
                let needs_eval = self.elapsed_time_in_global
                    || self
                        .current_state
                        .as_ref()
                        .map(|s| self.elapsed_time_states.contains(s.name()))
                        .unwrap_or(false);
                if needs_eval {
                    let _ = self.run_current_state_pipeline();
                }
            }
        }

        ticked
    }

    fn elapsed_time_increment(&mut self, dt: f32) {
        self.elapsed_time += (dt * 0.001).max(0.0);
    }

    pub fn get_inputs(&self) -> Vec<String> {
        let mut result = Vec::with_capacity((self.inputs.len() + 1) * 2);
        result.push(ELAPSED_TIME.to_string());
        result.push("Numeric".to_string());
        for name in self.inputs.numeric.keys() {
            result.push(name.as_str().to_owned());
            result.push("Numeric".to_string());
        }
        for name in self.inputs.boolean.keys() {
            result.push(name.as_str().to_owned());
            result.push("Boolean".to_string());
        }
        for name in self.inputs.string.keys() {
            result.push(name.as_str().to_owned());
            result.push("String".to_string());
        }
        for name in self.inputs.event.iter() {
            result.push(name.as_str().to_owned());
            result.push("Event".to_string());
        }
        result
    }
}

/// Returns:
///   - the set of PlaybackState names whose transitions reference `elapsedTime`
///   - whether the GlobalState's transitions reference `elapsedTime`
///
/// The GlobalState routes regardless of which PlaybackState is current, so the
/// per-tick gate must fire on every tick when it has elapsedTime guards —
/// independent of `current_state`.
fn compute_elapsed_time_states(state_machine: &StateMachine) -> (FxHashSet<DotString>, bool) {
    let mut set = FxHashSet::default();
    let mut in_global = false;
    for state in &state_machine.states {
        if guards_reference_elapsed_time(state.transitions()) {
            match state {
                State::GlobalState { .. } => in_global = true,
                State::PlaybackState { .. } => {
                    set.insert(state.name().clone());
                }
            }
        }
    }
    (set, in_global)
}

fn guards_reference_elapsed_time(transitions: &[Transition]) -> bool {
    for transition in transitions {
        let Some(guards) = transition.guards() else {
            continue;
        };
        for guard in guards {
            match guard {
                Guard::Numeric {
                    input_name,
                    compare_to,
                    ..
                } => {
                    if input_name == ELAPSED_TIME {
                        return true;
                    }
                    if let StringNumberBool::String(s) = compare_to {
                        if s == ELAPSED_TIME {
                            return true;
                        }
                    }
                }
                Guard::String { .. } | Guard::Boolean { .. } | Guard::Event { .. } => {}
            }
        }
    }
    false
}
