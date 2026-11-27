#![allow(clippy::print_stdout)]
//! State machine playground: run a state machine, drive its inputs and watch
//! the events it emits. Pointer events on the canvas are forwarded to it.
//!
//! ```text
//! cargo run --example state_machine                    # bundled demo
//! cargo run --example state_machine -- file.lottie     # state machines from the manifest
//! cargo run --example state_machine -- animation.lottie state_machine.json
//! ```

use dotlottie_rs::state_machine::OpenUrlPolicy;
use dotlottie_rs::Event;
use dotlottie_rs::{Player, Rgba, StateMachineEngine, StateMachineEvent};
use eframe::egui;
use std::collections::VecDeque;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

const MAX_LOG_ENTRIES: usize = 500;

#[derive(Clone, Debug)]
enum InputKind {
    Boolean(bool),
    Numeric(f32),
    StringInput(String),
    Event,
}

#[derive(Clone, Debug)]
struct InputEntry {
    name: String,
    kind: InputKind,
}

struct Playground {
    engine: Option<StateMachineEngine<'static>>,
    canvas: common::Canvas,
    inputs: Vec<InputEntry>,
    event_log: VecDeque<String>,
    loaded_file: String,
    pointer_entered: bool,
    last_mouse_pos: (f32, f32),
    auto_scroll_log: bool,
    available_state_machines: Vec<(String, String)>,
    selected_sm_index: Option<usize>,
    file_path: Option<PathBuf>,
}

impl Playground {
    fn new(file_path: PathBuf, state_machine_path: Option<PathBuf>) -> Self {
        let mut canvas = common::Canvas::new();
        let _ = canvas.player.set_background(Rgba::from(0xffffffff));

        let mut app = Self {
            engine: None,
            canvas,
            inputs: Vec::new(),
            event_log: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            loaded_file: String::new(),
            pointer_entered: false,
            last_mouse_pos: (0.0, 0.0),
            auto_scroll_log: true,
            available_state_machines: Vec::new(),
            selected_sm_index: None,
            file_path: None,
        };

        app.load_animation(file_path);
        if let Some(path) = &state_machine_path {
            app.load_state_machine_file(path);
        }

        app
    }

    fn load_animation(&mut self, path: PathBuf) {
        self.release_engine();
        self.available_state_machines.clear();
        self.selected_sm_index = None;

        if !common::load_file(&mut self.canvas.player, &path) {
            self.log(format!("ERROR: Failed to load {}", path.display()));
            return;
        }

        let name = common::file_name(&path);
        self.loaded_file = name.clone();
        self.file_path = Some(path);
        self.log(format!("Loaded: {name}"));
        self.discover_state_machines();
    }

    fn discover_state_machines(&mut self) {
        self.available_state_machines.clear();
        if let Some(manifest) = self.canvas.player.manifest() {
            if let Some(sms) = &manifest.state_machines {
                for sm in sms {
                    let display = sm.name.clone().unwrap_or_else(|| sm.id.clone());
                    self.available_state_machines.push((sm.id.clone(), display));
                }
            }
        }

        let count = self.available_state_machines.len();
        if count == 0 {
            self.log("No state machines packaged in this file".to_string());
        } else {
            self.log(format!("Found {count} state machine(s)"));
            if count == 1 {
                self.start_state_machine(0);
            }
        }
    }

    // A `.json` without `layers` is a state machine definition, not a Lottie.
    fn open_dropped(&mut self, path: PathBuf) {
        let is_state_machine = path.extension().is_some_and(|ext| ext == "json")
            && fs::read_to_string(&path).is_ok_and(|json| !json.contains("\"layers\""));

        if is_state_machine {
            self.load_state_machine_file(&path);
        } else {
            self.load_animation(path);
        }
    }

    fn start_state_machine(&mut self, index: usize) {
        let Some((id, display)) = self.available_state_machines.get(index).cloned() else {
            return;
        };
        let Ok(c_id) = CString::new(id) else {
            self.log("ERROR: Invalid state machine ID".to_string());
            return;
        };

        self.release_engine();
        let loaded = self.player_mut().state_machine_load(&c_id);
        if self.start_engine(loaded, &display) {
            self.selected_sm_index = Some(index);
        }
    }

    fn load_state_machine_file(&mut self, path: &Path) {
        let definition = match fs::read_to_string(path) {
            Ok(definition) => definition,
            Err(e) => {
                self.log(format!("ERROR: Cannot read {}: {e}", path.display()));
                return;
            }
        };

        self.release_engine();
        self.selected_sm_index = None;
        let loaded = self.player_mut().state_machine_load_data(&definition);
        self.start_engine(loaded, &common::file_name(path));
    }

    // The boxed player outlives the engine (see `Drop`).
    fn player_mut(&mut self) -> &'static mut Player {
        let player: *mut Player = &mut *self.canvas.player;
        unsafe { &mut *player }
    }

    fn start_engine<E: std::fmt::Debug>(
        &mut self,
        loaded: Result<StateMachineEngine<'static>, E>,
        name: &str,
    ) -> bool {
        let mut engine = match loaded {
            Ok(engine) => engine,
            Err(e) => {
                self.log(format!(
                    "ERROR: Failed to load state machine '{name}': {e:?}"
                ));
                return false;
            }
        };

        if let Err(e) = engine.start(&OpenUrlPolicy::default()) {
            self.log(format!("ERROR: Failed to start state machine: {e:?}"));
            engine.release();
            return false;
        }

        self.discover_inputs(&engine);
        self.engine = Some(engine);
        self.log(format!("Started: {name}"));
        true
    }

    fn release_engine(&mut self) {
        if let Some(engine) = self.engine.take() {
            engine.release();
            self.inputs.clear();
            self.log("State machine released".to_string());
        }
    }

    fn discover_inputs(&mut self, engine: &StateMachineEngine) {
        self.inputs.clear();
        let raw = engine.get_inputs();
        for pair in raw.chunks(2) {
            if pair.len() < 2 {
                break;
            }
            let name = pair[0].clone();
            let kind = match pair[1].as_str() {
                "Boolean" => {
                    let val = engine.get_boolean_input(&name).unwrap_or(false);
                    InputKind::Boolean(val)
                }
                "Numeric" => {
                    let val = engine.get_numeric_input(&name).unwrap_or(0.0);
                    InputKind::Numeric(val)
                }
                "String" => {
                    let val = engine.get_string_input(&name).unwrap_or_default();
                    InputKind::StringInput(val)
                }
                "Event" => InputKind::Event,
                _ => continue,
            };
            self.inputs.push(InputEntry { name, kind });
        }
    }

    fn refresh_input_values(&mut self) {
        let Some(engine) = &self.engine else { return };
        for input in &mut self.inputs {
            match &mut input.kind {
                InputKind::Boolean(val) => {
                    if let Some(v) = engine.get_boolean_input(&input.name) {
                        *val = v;
                    }
                }
                InputKind::Numeric(val) => {
                    if let Some(v) = engine.get_numeric_input(&input.name) {
                        *val = v;
                    }
                }
                InputKind::StringInput(val) => {
                    if let Some(v) = engine.get_string_input(&input.name) {
                        *val = v;
                    }
                }
                InputKind::Event => {}
            }
        }
    }

    fn poll_events(&mut self) {
        let Some(engine) = &mut self.engine else {
            return;
        };
        while let Some(event) = engine.poll_event() {
            let msg = format_event(&event);
            println!("{msg}");
            self.event_log.push_back(msg);
            if self.event_log.len() > MAX_LOG_ENTRIES {
                self.event_log.pop_front();
            }
        }
    }

    fn log(&mut self, msg: String) {
        println!("{msg}");
        self.event_log.push_back(msg);
        if self.event_log.len() > MAX_LOG_ENTRIES {
            self.event_log.pop_front();
        }
    }
}

impl Drop for Playground {
    fn drop(&mut self) {
        self.release_engine();
    }
}

impl eframe::App for Playground {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dropped: Vec<PathBuf> = ui.ctx().input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        for path in dropped {
            self.open_dropped(path);
        }

        self.poll_events();

        let dt = self.canvas.dt();

        if let Some(engine) = &mut self.engine {
            let _ = engine.tick(dt);
        }

        self.refresh_input_values();

        egui::Panel::right("controls")
            .default_size(280.0)
            .show(ui, |ui| {
                ui.heading("State Machine Playground");
                ui.separator();

                ui.label("Drop an animation (.lottie, .json) or a state machine .json here");

                ui.horizontal(|ui| {
                    if !self.loaded_file.is_empty() {
                        ui.label(format!("File: {}", self.loaded_file));
                    }
                    if let Some(path) = self.file_path.clone() {
                        if ui.button("Reload").clicked() {
                            self.load_animation(path);
                        }
                    }
                });

                if let Some(engine) = &self.engine {
                    ui.label(format!("Status: {}", engine.status()));
                    ui.label(format!("State: {}", engine.get_current_state_name()));
                }

                ui.separator();

                if !self.available_state_machines.is_empty() {
                    ui.label("State Machines:");
                    let mut start_index: Option<usize> = None;
                    for (i, (_id, display)) in self.available_state_machines.iter().enumerate() {
                        let is_selected = self.selected_sm_index == Some(i);
                        ui.horizontal(|ui| {
                            let label = if is_selected {
                                format!(">> {display}")
                            } else {
                                display.clone()
                            };
                            if ui.button(&label).clicked() && !is_selected {
                                start_index = Some(i);
                            }
                        });
                    }
                    if let Some(idx) = start_index {
                        self.start_state_machine(idx);
                    }
                    ui.separator();
                }

                if !self.inputs.is_empty() {
                    ui.heading("Inputs");

                    let mut changes: Vec<(usize, InputKind)> = Vec::new();
                    let mut fires: Vec<String> = Vec::new();

                    for (i, input) in self.inputs.iter_mut().enumerate() {
                        ui.horizontal(|ui| match &mut input.kind {
                            InputKind::Boolean(val) => {
                                let prev = *val;
                                ui.checkbox(val, &input.name);
                                if *val != prev {
                                    changes.push((i, InputKind::Boolean(*val)));
                                }
                            }
                            InputKind::Numeric(val) => {
                                ui.label(&input.name);
                                let prev = *val;
                                ui.add(
                                    egui::Slider::new(val, -100.0..=100.0)
                                        .clamping(egui::SliderClamping::Never),
                                );
                                if (*val - prev).abs() > f32::EPSILON {
                                    changes.push((i, InputKind::Numeric(*val)));
                                }
                            }
                            InputKind::StringInput(val) => {
                                ui.label(&input.name);
                                let prev = val.clone();
                                ui.text_edit_singleline(val);
                                if *val != prev {
                                    changes.push((i, InputKind::StringInput(val.clone())));
                                }
                            }
                            InputKind::Event => {
                                if ui.button(format!("Fire: {}", input.name)).clicked() {
                                    fires.push(input.name.clone());
                                }
                            }
                        });
                    }

                    if let Some(engine) = &mut self.engine {
                        for (idx, kind) in changes {
                            let name = &self.inputs[idx].name;
                            match kind {
                                InputKind::Boolean(v) => {
                                    let _ = engine.set_boolean_input(name, v, true, false);
                                }
                                InputKind::Numeric(v) => {
                                    let _ = engine.set_numeric_input(name, v, true, false);
                                }
                                InputKind::StringInput(v) => {
                                    let _ = engine.set_string_input(name, &v, true, false);
                                }
                                InputKind::Event => {}
                            }
                        }
                        for name in fires {
                            let _ = engine.fire(&name, true);
                        }
                    }

                    ui.separator();
                }

                ui.collapsing("Manual Events", |ui| {
                    let (mx, my) = self.last_mouse_pos;
                    ui.label(format!("Mouse pos: ({mx:.0}, {my:.0})"));

                    type EventFactory = fn(f32, f32) -> Event;
                    let events_to_post: &[(&str, EventFactory)] = &[
                        ("PointerDown", |x, y| Event::PointerDown { x, y }),
                        ("PointerUp", |x, y| Event::PointerUp { x, y }),
                        ("PointerMove", |x, y| Event::PointerMove { x, y }),
                        ("PointerEnter", |x, y| Event::PointerEnter { x, y }),
                        ("PointerExit", |x, y| Event::PointerExit { x, y }),
                        ("Click", |x, y| Event::Click { x, y }),
                    ];

                    for (label, make_event) in events_to_post {
                        if ui.button(*label).clicked() {
                            if let Some(engine) = &mut self.engine {
                                engine.post_event(&make_event(mx, my));
                            }
                        }
                    }

                    if ui.button("OnComplete").clicked() {
                        if let Some(engine) = &mut self.engine {
                            engine.post_event(&Event::OnComplete);
                        }
                    }
                    if ui.button("OnLoopComplete").clicked() {
                        if let Some(engine) = &mut self.engine {
                            engine.post_event(&Event::OnLoopComplete);
                        }
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.heading("Event Log");
                    if ui.button("Clear").clicked() {
                        self.event_log.clear();
                    }
                    ui.checkbox(&mut self.auto_scroll_log, "Auto-scroll");
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(self.auto_scroll_log)
                    .show(ui, |ui| {
                        for entry in &self.event_log {
                            ui.label(entry);
                        }
                    });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.file_path.is_some() {
                let response = self.canvas.show(ui);

                if let Some(engine) = &mut self.engine {
                    let pointer_pos = response
                        .interact_pointer_pos()
                        .or_else(|| response.hover_pos());

                    if let Some(pos) = pointer_pos {
                        let local = common::Canvas::to_canvas(pos, response.rect);
                        self.last_mouse_pos = local;
                    }

                    let (mx, my) = self.last_mouse_pos;
                    let hovered = response.hovered() || response.is_pointer_button_down_on();

                    if hovered && !self.pointer_entered {
                        engine.post_event(&Event::PointerEnter { x: mx, y: my });
                        self.pointer_entered = true;
                    } else if !hovered && self.pointer_entered {
                        engine.post_event(&Event::PointerExit { x: mx, y: my });
                        self.pointer_entered = false;
                    }

                    if hovered && !response.is_pointer_button_down_on() {
                        engine.post_event(&Event::PointerMove { x: mx, y: my });
                    }

                    if response.drag_started_by(egui::PointerButton::Primary) {
                        engine.post_event(&Event::PointerDown { x: mx, y: my });
                    }

                    if response.drag_stopped() {
                        engine.post_event(&Event::PointerUp { x: mx, y: my });
                        engine.post_event(&Event::Click { x: mx, y: my });
                    }

                    if response.clicked() && !response.drag_stopped() {
                        engine.post_event(&Event::PointerDown { x: mx, y: my });
                        engine.post_event(&Event::PointerUp { x: mx, y: my });
                        engine.post_event(&Event::Click { x: mx, y: my });
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No animation loaded. Drop a .lottie or .json file here.");
                });
            }
        });
    }
}

fn format_event(event: &StateMachineEvent) -> String {
    match event {
        StateMachineEvent::Start => "[SM] Start".to_string(),
        StateMachineEvent::Stop => "[SM] Stop".to_string(),
        StateMachineEvent::Transition {
            previous_state,
            new_state,
        } => format!("[SM] Transition: {previous_state} -> {new_state}"),
        StateMachineEvent::StateEntered { state } => {
            format!("[SM] StateEntered: {state}")
        }
        StateMachineEvent::StateExit { state } => {
            format!("[SM] StateExit: {state}")
        }
        StateMachineEvent::CustomEvent { message } => {
            format!("[SM] CustomEvent: {message}")
        }
        StateMachineEvent::Error { message } => {
            format!("[SM] Error: {message}")
        }
        StateMachineEvent::StringInputChange {
            name,
            old_value,
            new_value,
        } => format!("[Input] {name} (string): {old_value} -> {new_value}"),
        StateMachineEvent::NumericInputChange {
            name,
            old_value,
            new_value,
        } => format!("[Input] {name} (numeric): {old_value} -> {new_value}"),
        StateMachineEvent::BooleanInputChange {
            name,
            old_value,
            new_value,
        } => format!("[Input] {name} (bool): {old_value} -> {new_value}"),
        StateMachineEvent::InputFired { name } => {
            format!("[Input] Fired: {name}")
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(PathBuf::from);
    let file_path = args.next().unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/animations/dotlottie/v2/pigeon.lottie")
    });
    let state_machine_path = args.next();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "dotLottie State Machine Playground",
        native_options,
        Box::new(move |_cc| Ok(Box::new(Playground::new(file_path, state_machine_path)))),
    )
    .expect("Failed to run eframe");
}
