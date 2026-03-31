#![allow(clippy::print_stdout)]

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, DotLottiePlayer, StateMachineEngine, StateMachineEvent};
use eframe::egui;
use std::collections::VecDeque;
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

const CANVAS_WIDTH: u32 = 500;
const CANVAS_HEIGHT: u32 = 500;
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
    player: Box<DotLottiePlayer>,
    buffer: Vec<u32>,
    engine: Option<StateMachineEngine<'static>>,
    texture: Option<egui::TextureHandle>,
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
    fn new(file_path: Option<PathBuf>) -> Self {
        let mut player = Box::new(DotLottiePlayer::new());
        let _ = player.set_background_color(Some(0xffffffff));

        let mut buffer = vec![0u32; (CANVAS_WIDTH * CANVAS_HEIGHT) as usize];
        player
            .set_sw_target(
                &mut buffer,
                CANVAS_WIDTH,
                CANVAS_HEIGHT,
                ColorSpace::ABGR8888,
            )
            .expect("Failed to set SW target");

        let mut app = Self {
            player,
            buffer,
            engine: None,
            texture: None,
            inputs: Vec::new(),
            event_log: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            loaded_file: String::new(),
            pointer_entered: false,
            last_mouse_pos: (0.0, 0.0),
            auto_scroll_log: true,
            available_state_machines: Vec::new(),
            selected_sm_index: None,
            file_path: file_path.clone(),
        };

        if let Some(path) = &file_path {
            app.load_dotlottie(path.clone());
        }

        app
    }

    fn load_dotlottie(&mut self, path: PathBuf) {
        self.release_engine();
        self.available_state_machines.clear();
        self.selected_sm_index = None;

        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                self.log(format!("ERROR: Cannot open file: {e}"));
                return;
            }
        };
        let metadata = fs::metadata(&path).unwrap();
        let mut data = vec![0u8; metadata.len() as usize];
        file.read_exact(&mut data).unwrap();

        match self.player.load_dotlottie_data(&data, CANVAS_WIDTH, CANVAS_HEIGHT) {
            Ok(_) => {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                self.loaded_file = name.clone();
                self.file_path = Some(path);
                self.log(format!("Loaded: {name}"));
                self.discover_state_machines();
            }
            Err(_) => {
                self.log("ERROR: Failed to load .lottie file".to_string());
            }
        }
    }

    fn discover_state_machines(&mut self) {
        self.available_state_machines.clear();
        if let Some(manifest) = self.player.manifest() {
            if let Some(sms) = &manifest.state_machines {
                for sm in sms {
                    let display = sm.name.clone().unwrap_or_else(|| sm.id.clone());
                    self.available_state_machines.push((sm.id.clone(), display));
                }
            }
        }

        let count = self.available_state_machines.len();
        if count == 0 {
            self.log("No state machines found in this .lottie file".to_string());
        } else {
            self.log(format!("Found {count} state machine(s)"));
            if count == 1 {
                self.start_state_machine(0);
            }
        }
    }

    fn start_state_machine(&mut self, index: usize) {
        self.release_engine();

        let (id, display) = match self.available_state_machines.get(index) {
            Some(sm) => sm.clone(),
            None => return,
        };

        let player_ptr: *mut DotLottiePlayer = &mut *self.player;
        let player_ref: &'static mut DotLottiePlayer = unsafe { &mut *player_ptr };

        let c_id = match CString::new(id.clone()) {
            Ok(c) => c,
            Err(e) => {
                self.log(format!("ERROR: Invalid state machine ID: {e}"));
                return;
            }
        };

        match player_ref.state_machine_load(&c_id) {
            Ok(mut engine) => {
                let open_url = OpenUrlPolicy::default();
                match engine.start(&open_url) {
                    Ok(()) => {
                        self.selected_sm_index = Some(index);
                        self.discover_inputs(&engine);
                        self.engine = Some(engine);
                        self.log(format!("Started: {display}"));
                    }
                    Err(e) => {
                        self.log(format!("ERROR: Failed to start state machine: {e:?}"));
                        engine.release();
                    }
                }
            }
            Err(e) => {
                self.log(format!("ERROR: Failed to load state machine '{id}': {e:?}"));
            }
        }
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

    fn buffer_to_color_image(&self) -> egui::ColorImage {
        let w = CANVAS_WIDTH as usize;
        let h = CANVAS_HEIGHT as usize;
        let pixels: Vec<egui::Color32> = self
            .buffer
            .iter()
            .map(|&abgr| {
                let a = ((abgr >> 24) & 0xFF) as u8;
                let b = ((abgr >> 16) & 0xFF) as u8;
                let g = ((abgr >> 8) & 0xFF) as u8;
                let r = (abgr & 0xFF) as u8;
                egui::Color32::from_rgba_premultiplied(r, g, b, a)
            })
            .collect();
        egui::ColorImage {
            size: [w, h],
            pixels,
        }
    }
}

impl eframe::App for Playground {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        self.poll_events();

        if let Some(engine) = &mut self.engine {
            let _ = engine.tick();
        }

        self.refresh_input_values();

        let image = self.buffer_to_color_image();
        match &mut self.texture {
            Some(tex) => tex.set(image, egui::TextureOptions::NEAREST),
            None => {
                self.texture =
                    Some(ctx.load_texture("animation", image, egui::TextureOptions::NEAREST));
            }
        }

        ctx.request_repaint();

        egui::SidePanel::right("controls")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("State Machine Playground");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Open .lottie").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("dotLottie", &["lottie"])
                            .pick_file()
                        {
                            self.load_dotlottie(path);
                        }
                    }
                    if self.file_path.is_some() {
                        if ui.button("Reload").clicked() {
                            let path = self.file_path.clone().unwrap();
                            self.load_dotlottie(path);
                        }
                    }
                });

                if !self.loaded_file.is_empty() {
                    ui.label(format!("File: {}", self.loaded_file));
                }

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
                        ui.horizontal(|ui| {
                            match &mut input.kind {
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

                    let events_to_post: &[(&str, fn(f32, f32) -> Event)] = &[
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
                    if ui.button("Copy").clicked() {
                        let text: String = self.event_log.iter().fold(
                            String::new(),
                            |mut acc, entry| {
                                acc.push_str(entry);
                                acc.push('\n');
                                acc
                            },
                        );
                        ui.ctx().copy_text(text);
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

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture {
                let available = ui.available_size();
                let scale = (available.x / CANVAS_WIDTH as f32)
                    .min(available.y / CANVAS_HEIGHT as f32)
                    .min(1.0);
                let display_size =
                    egui::vec2(CANVAS_WIDTH as f32 * scale, CANVAS_HEIGHT as f32 * scale);

                let (rect, response) =
                    ui.allocate_exact_size(display_size, egui::Sense::click_and_drag());

                ui.painter().image(
                    tex.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                ui.painter().rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                    egui::StrokeKind::Outside,
                );

                if let Some(engine) = &mut self.engine {
                    let pointer_pos = response
                        .interact_pointer_pos()
                        .or_else(|| response.hover_pos());

                    if let Some(pos) = pointer_pos {
                        let local = to_canvas_coords(pos, rect, scale);
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
                    ui.label(
                        "No animation loaded.\nClick 'Open .lottie' to load a dotLottie file.",
                    );
                });
            }
        });
    }
}

impl Drop for Playground {
    fn drop(&mut self) {
        self.release_engine();
    }
}

fn to_canvas_coords(screen_pos: egui::Pos2, rect: egui::Rect, scale: f32) -> (f32, f32) {
    let x = (screen_pos.x - rect.min.x) / scale;
    let y = (screen_pos.y - rect.min.y) / scale;
    (
        x.clamp(0.0, CANVAS_WIDTH as f32),
        y.clamp(0.0, CANVAS_HEIGHT as f32),
    )
}

fn format_event(event: &StateMachineEvent) -> String {
    match event {
        StateMachineEvent::Start => "[SM] Start".to_string(),
        StateMachineEvent::Stop => "[SM] Stop".to_string(),
        StateMachineEvent::Transition {
            previous_state,
            new_state,
        } => format!(
            "[SM] Transition: {} -> {}",
            previous_state.to_string_lossy(),
            new_state.to_string_lossy()
        ),
        StateMachineEvent::StateEntered { state } => {
            format!("[SM] StateEntered: {}", state.to_string_lossy())
        }
        StateMachineEvent::StateExit { state } => {
            format!("[SM] StateExit: {}", state.to_string_lossy())
        }
        StateMachineEvent::CustomEvent { message } => {
            format!("[SM] CustomEvent: {}", message.to_string_lossy())
        }
        StateMachineEvent::Error { message } => {
            format!("[SM] Error: {}", message.to_string_lossy())
        }
        StateMachineEvent::StringInputChange {
            name,
            old_value,
            new_value,
        } => format!(
            "[Input] {} (string): {} -> {}",
            name.to_string_lossy(),
            old_value.to_string_lossy(),
            new_value.to_string_lossy()
        ),
        StateMachineEvent::NumericInputChange {
            name,
            old_value,
            new_value,
        } => format!(
            "[Input] {} (numeric): {old_value} -> {new_value}",
            name.to_string_lossy()
        ),
        StateMachineEvent::BooleanInputChange {
            name,
            old_value,
            new_value,
        } => format!(
            "[Input] {} (bool): {old_value} -> {new_value}",
            name.to_string_lossy()
        ),
        StateMachineEvent::InputFired { name } => {
            format!("[Input] Fired: {}", name.to_string_lossy())
        }
    }
}

fn main() {
    let file_path = std::env::args().nth(1).map(PathBuf::from);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "dotLottie State Machine Playground",
        native_options,
        Box::new(move |_cc| Ok(Box::new(Playground::new(file_path)))),
    )
    .expect("Failed to run eframe");
}
