use std::sync::RwLock;
use std::{rc::Rc, sync::Arc};

use crate::fms::Manifest;
use crate::lottie_renderer::Renderer;
use crate::markers::Marker;
use crate::state_machine::errors::StateMachineError::ParsingError;
use crate::state_machine::events::Event;
use crate::state_machine::listeners::ListenerTrait;
use crate::state_machine::{StateMachine, StateMachineObserver, StateMachineStatus};

use crate::dotlottie_player::Config;
use crate::dotlottie_player::DotLottiePlayer as DotLottieRuntime;
use crate::dotlottie_player::Observer;

pub struct DotLottiePlayerContainer {
    runtime: RwLock<DotLottieRuntime>,
    observers: RwLock<Vec<Arc<dyn Observer>>>,
    state_machine: Rc<RwLock<Option<StateMachine>>>,
}

impl DotLottiePlayerContainer {
    #[cfg(feature = "thorvg")]
    pub fn new(config: Config) -> Self {
        DotLottiePlayerContainer {
            runtime: RwLock::new(DotLottieRuntime::new(config)),
            observers: RwLock::new(Vec::new()),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        DotLottiePlayerContainer {
            runtime: RwLock::new(DotLottieRuntime::with_renderer(config, renderer)),
            observers: RwLock::new(Vec::new()),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn load_animation_data(&self, animation_data: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation_data(animation_data, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_animation_path(&self, animation_path: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation_path(animation_path, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_dotlottie_data(&self, file_data: &[u8], width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_dotlottie_data(file_data, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_animation(&self, animation_id: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation(animation_id, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn manifest(&self) -> Option<Manifest> {
        self.runtime
            .read()
            .ok()
            .and_then(|runtime| runtime.manifest().cloned())
    }

    pub fn buffer(&self) -> *const u32 {
        self.runtime.read().unwrap().buffer().as_ptr()
    }

    pub fn buffer_ptr(&self) -> u64 {
        self.runtime.read().unwrap().buffer().as_ptr().cast::<u32>() as u64
    }

    pub fn buffer_len(&self) -> u64 {
        self.runtime.read().unwrap().buffer().len() as u64
    }

    pub fn clear(&self) {
        self.runtime.write().unwrap().clear();
    }

    pub fn set_config(&self, config: Config) {
        self.runtime.write().unwrap().set_config(config);
    }

    pub fn size(&self) -> (u32, u32) {
        self.runtime.read().unwrap().size()
    }

    pub fn speed(&self) -> f32 {
        self.runtime.read().unwrap().speed()
    }

    pub fn total_frames(&self) -> f32 {
        self.runtime.read().unwrap().total_frames()
    }

    pub fn duration(&self) -> f32 {
        self.runtime.read().unwrap().duration()
    }

    pub fn segment_duration(&self) -> f32 {
        self.runtime.read().unwrap().segment_duration()
    }

    pub fn current_frame(&self) -> f32 {
        self.runtime.read().unwrap().current_frame()
    }

    pub fn loop_count(&self) -> u32 {
        self.runtime.read().unwrap().loop_count()
    }

    pub fn is_loaded(&self) -> bool {
        self.runtime.read().unwrap().is_loaded()
    }

    pub fn is_playing(&self) -> bool {
        self.runtime.read().unwrap().is_playing()
    }

    pub fn is_paused(&self) -> bool {
        self.runtime.read().unwrap().is_paused()
    }

    pub fn is_stopped(&self) -> bool {
        self.runtime.read().unwrap().is_stopped()
    }

    pub fn play(&self) -> bool {
        let ok = self.runtime.write().unwrap().play();

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_play();
            });
        }

        ok
    }

    pub fn pause(&self) -> bool {
        let ok = self.runtime.write().unwrap().pause();

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_pause();
            });
        }

        ok
    }

    pub fn stop(&self) -> bool {
        let ok = self.runtime.write().unwrap().stop();

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_stop();
            });
        }

        ok
    }

    pub fn request_frame(&self) -> f32 {
        self.runtime.write().unwrap().request_frame()
    }

    pub fn set_frame(&self, no: f32) -> bool {
        let ok = self.runtime.write().unwrap().set_frame(no);

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_frame(no);
            });
        }

        ok
    }

    pub fn seek(&self, no: f32) -> bool {
        let ok = self.runtime.write().unwrap().seek(no);

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_frame(no);
            });
        }

        ok
    }

    pub fn render(&self) -> bool {
        let ok = self.runtime.write().unwrap().render();

        if ok {
            let frame_no = self.current_frame();

            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_render(frame_no);
            });

            if self.is_complete() {
                if self.config().loop_animation {
                    self.observers.read().unwrap().iter().for_each(|observer| {
                        observer.on_loop(self.loop_count());
                    });
                } else {
                    self.observers.read().unwrap().iter().for_each(|observer| {
                        observer.on_complete();
                    });

                    if let Ok(mut state_machine) = self.state_machine.try_write() {
                        if let Some(sm) = state_machine.as_mut() {
                            sm.post_event(&Event::OnComplete);
                        }
                    }
                }
            }
        }

        ok
    }

    pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        match self.runtime.try_write() {
            Ok(mut runtime) => runtime.set_viewport(x, y, w, h),
            _ => false,
        }
    }

    pub fn resize(&self, width: u32, height: u32) -> bool {
        self.runtime.write().unwrap().resize(width, height)
    }

    pub fn config(&self) -> Config {
        self.runtime.read().unwrap().config()
    }

    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        self.observers.write().unwrap().push(observer);
    }

    pub fn manifest_string(&self) -> String {
        self.runtime
            .try_read()
            .ok()
            .and_then(|runtime| runtime.manifest().cloned())
            .map_or_else(String::new, |manifest| {
                serde_json::to_string(&manifest).unwrap()
            })
    }

    pub fn is_complete(&self) -> bool {
        self.runtime.read().unwrap().is_complete()
    }

    pub fn unsubscribe(&self, observer: &Arc<dyn Observer>) {
        self.observers
            .write()
            .unwrap()
            .retain(|o| !Arc::ptr_eq(o, observer));
    }

    pub fn set_theme(&self, theme_id: &str) -> bool {
        self.runtime.write().unwrap().set_theme(theme_id)
    }

    pub fn reset_theme(&self) -> bool {
        self.runtime.write().unwrap().reset_theme()
    }

    pub fn set_theme_data(&self, theme_data: &str) -> bool {
        self.runtime.write().unwrap().set_theme_data(theme_data)
    }

    pub fn set_slots(&self, slots: &str) -> bool {
        self.runtime.write().unwrap().set_slots(slots)
    }

    pub fn animation_size(&self) -> Vec<f32> {
        let (width, height) = self.runtime.read().unwrap().animation_size();
        vec![width, height]
    }

    pub fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> bool {
        self.runtime.read().unwrap().hit_check(layer_name, x, y)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.runtime.read().unwrap().get_layer_bounds(layer_name)
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.runtime.read().unwrap().markers()
    }

    pub fn active_animation_id(&self) -> String {
        self.runtime
            .read()
            .unwrap()
            .active_animation_id()
            .to_string()
    }

    pub fn active_theme_id(&self) -> String {
        self.runtime.read().unwrap().active_theme_id().to_string()
    }

    pub fn get_state_machine(&self, state_machine_id: &str) -> Option<String> {
        match self.runtime.try_read() {
            Ok(runtime) => runtime.get_state_machine(state_machine_id),
            Err(_) => None,
        }
    }
}

pub struct DotLottiePlayer {
    player: Rc<RwLock<DotLottiePlayerContainer>>,
    state_machine: Rc<RwLock<Option<StateMachine>>>,
}

impl DotLottiePlayer {
    #[cfg(feature = "thorvg")]
    pub fn new(config: Config) -> Self {
        DotLottiePlayer {
            player: Rc::new(RwLock::new(DotLottiePlayerContainer::new(config))),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        DotLottiePlayer {
            player: Rc::new(RwLock::new(DotLottiePlayerContainer::with_renderer(
                config, renderer,
            ))),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn load_animation_data(&self, animation_data: &str, width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_animation_data(animation_data, width, height))
    }

    pub fn get_state_machine(&self) -> Rc<RwLock<Option<StateMachine>>> {
        self.state_machine.clone()
    }

    pub fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> bool {
        self.player.read().unwrap().hit_check(layer_name, x, y)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.player.read().unwrap().get_layer_bounds(layer_name)
    }

    // If you are in an environment that does not support events
    // Call isPlaying() to know if the state machine started playback within the first state
    pub fn start_state_machine(&self) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => {
                return false;
            }
        }

        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.start();
                }
            }
            Err(_) => {
                return false;
            }
        }

        true
    }

    pub fn stop_state_machine(&self) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    if sm.status == StateMachineStatus::Running {
                        sm.end();
                    } else {
                        return false;
                    }
                }
            }
            Err(_) => return false,
        }

        true
    }

    /// Returns which types of listeners need to be setup.
    /// The frameworks should call the function after calling start_state_machine.
    pub fn state_machine_framework_setup(&self) -> Vec<String> {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return vec![];
                }

                let mut listener_types = vec![];

                if let Some(sm) = state_machine.as_ref() {
                    let listeners = sm.get_listeners();

                    for listener in listeners {
                        match listener.try_read() {
                            Ok(listener) => {
                                if !listener_types.contains(&listener.get_type().to_string()) {
                                    listener_types.push(listener.get_type().to_string());
                                }
                            }
                            Err(_) => return vec![],
                        }
                    }
                    listener_types
                } else {
                    vec![]
                }
            }
            Err(_) => vec![],
        }
    }

    pub fn set_state_machine_numeric_context(&self, key: &str, value: f32) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        let sm_write = self.state_machine.try_write();

        match sm_write {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.set_numeric_context(key, value);
                }
            }
            Err(_) => return false,
        }

        true
    }

    pub fn set_state_machine_string_context(&self, key: &str, value: &str) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        let sm_write = self.state_machine.try_write();

        match sm_write {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.set_string_context(key, value);
                }
            }
            Err(_) => return false,
        }

        true
    }

    pub fn set_state_machine_boolean_context(&self, key: &str, value: bool) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        let sm_write = self.state_machine.try_write();

        match sm_write {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.set_bool_context(key, value);
                }
            }
            Err(_) => return false,
        }

        true
    }

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn post_event(&self, event: &Event) -> i32 {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return 1;
                }
            }
            Err(_) => return 1,
        }

        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    return sm.post_event(event);
                }
            }
            Err(_) => return 1,
        }

        1
    }

    pub fn post_bool_event(&self, value: bool) -> i32 {
        let event = Event::Bool { value };
        self.post_event(&event)
    }

    pub fn post_string_event(&self, value: &str) -> i32 {
        let event = Event::String {
            value: value.to_string(),
        };
        self.post_event(&event)
    }

    pub fn post_numeric_event(&self, value: f32) -> i32 {
        let event = Event::Numeric { value };
        self.post_event(&event)
    }

    pub fn post_pointer_down_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerDown { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_up_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerUp { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_move_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerMove { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_enter_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerEnter { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_exit_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerExit { x, y };
        self.post_event(&event)
    }

    pub fn post_set_numeric_context(&self, key: &str, value: f32) -> i32 {
        let event = Event::SetNumericContext {
            key: key.to_string(),
            value,
        };

        self.post_event(&event)
    }

    pub fn load_animation_path(&self, animation_path: &str, width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_animation_path(animation_path, width, height))
    }

    pub fn load_dotlottie_data(&self, file_data: &[u8], width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_dotlottie_data(file_data, width, height))
    }

    pub fn load_animation(&self, animation_id: &str, width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_animation(animation_id, width, height))
    }

    pub fn manifest(&self) -> Option<Manifest> {
        self.player.read().unwrap().manifest()
    }

    pub fn buffer(&self) -> *const u32 {
        self.player.read().unwrap().buffer()
    }

    pub fn buffer_ptr(&self) -> u64 {
        self.player.read().unwrap().buffer_ptr()
    }

    pub fn buffer_len(&self) -> u64 {
        self.player.read().unwrap().buffer_len()
    }

    pub fn clear(&self) {
        self.player.write().unwrap().clear();
    }

    pub fn set_config(&self, config: Config) {
        self.player.write().unwrap().set_config(config);
    }

    pub fn speed(&self) -> f32 {
        self.player.read().unwrap().speed()
    }

    pub fn total_frames(&self) -> f32 {
        self.player.read().unwrap().total_frames()
    }

    pub fn duration(&self) -> f32 {
        self.player.read().unwrap().duration()
    }

    pub fn current_frame(&self) -> f32 {
        self.player.read().unwrap().current_frame()
    }

    pub fn loop_count(&self) -> u32 {
        self.player.read().unwrap().loop_count()
    }

    pub fn is_loaded(&self) -> bool {
        self.player.read().unwrap().is_loaded()
    }

    pub fn is_playing(&self) -> bool {
        self.player.read().unwrap().is_playing()
    }

    pub fn is_paused(&self) -> bool {
        self.player.read().unwrap().is_paused()
    }

    pub fn is_stopped(&self) -> bool {
        self.player.read().unwrap().is_stopped()
    }

    pub fn segment_duration(&self) -> f32 {
        self.player.read().unwrap().segment_duration()
    }

    pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.player.write().unwrap().set_viewport(x, y, w, h)
    }

    pub fn play(&self) -> bool {
        self.player.write().unwrap().play()
    }

    pub fn pause(&self) -> bool {
        self.player.write().unwrap().pause()
    }

    pub fn stop(&self) -> bool {
        self.player.write().unwrap().stop()
    }

    pub fn request_frame(&self) -> f32 {
        self.player.write().unwrap().request_frame()
    }

    pub fn set_frame(&self, no: f32) -> bool {
        self.player.write().unwrap().set_frame(no)
    }

    pub fn seek(&self, no: f32) -> bool {
        self.player.write().unwrap().seek(no)
    }

    pub fn render(&self) -> bool {
        self.player.read().unwrap().render()
    }

    pub fn resize(&self, width: u32, height: u32) -> bool {
        self.player.write().unwrap().resize(width, height)
    }

    pub fn config(&self) -> Config {
        self.player.read().unwrap().config()
    }

    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        self.player.write().unwrap().subscribe(observer);
    }

    pub fn state_machine_subscribe(&self, observer: Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }
        sm.as_mut().unwrap().subscribe(observer);

        true
    }

    pub fn state_machine_unsubscribe(&self, observer: Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }

        sm.as_mut().unwrap().unsubscribe(&observer);

        true
    }

    pub fn manifest_string(&self) -> String {
        self.player
            .try_read()
            .map_or_else(|_| String::new(), |player| player.manifest_string())
    }

    pub fn is_complete(&self) -> bool {
        self.player.read().unwrap().is_complete()
    }

    pub fn unsubscribe(&self, observer: &Arc<dyn Observer>) {
        self.player.write().unwrap().unsubscribe(observer);
    }

    pub fn set_theme(&self, theme_id: &str) -> bool {
        self.player.write().unwrap().set_theme(theme_id)
    }

    pub fn reset_theme(&self) -> bool {
        self.player.write().unwrap().reset_theme()
    }

    pub fn load_state_machine_data(&self, state_machine: &str) -> bool {
        let state_machine = StateMachine::new(state_machine, self.player.clone());

        if state_machine.is_ok() {
            match self.state_machine.try_write() {
                Ok(mut sm) => {
                    sm.replace(state_machine.unwrap());
                }
                Err(_) => {
                    return false;
                }
            }

            let player = self.player.try_write();

            match player {
                Ok(mut player) => {
                    player.state_machine = self.state_machine.clone();
                }
                Err(_) => {
                    return false;
                }
            }
        }

        true
    }

    pub fn load_state_machine(&self, state_machine_id: &str) -> bool {
        let state_machine_string = self
            .player
            .read()
            .unwrap()
            .get_state_machine(state_machine_id);

        match state_machine_string {
            Some(machine) => {
                let state_machine = StateMachine::new(&machine, self.player.clone());

                if state_machine.is_ok() {
                    match self.state_machine.try_write() {
                        Ok(mut sm) => {
                            sm.replace(state_machine.unwrap());
                        }
                        Err(_) => {
                            return false;
                        }
                    }

                    let player = self.player.try_write();

                    match player {
                        Ok(mut player) => {
                            player.state_machine = self.state_machine.clone();
                        }
                        Err(_) => {
                            return false;
                        }
                    }
                } else if let Err(ParsingError { reason: _ }) = state_machine {
                    return false;
                }
            }
            None => {
                return false;
            }
        }
        true
    }

    pub fn set_theme_data(&self, theme_data: &str) -> bool {
        self.player.write().unwrap().set_theme_data(theme_data)
    }

    pub fn set_slots(&self, slots: &str) -> bool {
        self.player.write().unwrap().set_slots(slots)
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.player.read().unwrap().markers()
    }

    pub fn active_animation_id(&self) -> String {
        self.player
            .read()
            .unwrap()
            .active_animation_id()
            .to_string()
    }

    pub fn active_theme_id(&self) -> String {
        self.player.read().unwrap().active_theme_id().to_string()
    }

    pub fn animation_size(&self) -> Vec<f32> {
        self.player.read().unwrap().animation_size()
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}

pub mod prelude {
    pub use super::*;
    pub use crate::dotlottie_player::{Config, LayerBoundingBox, Mode, Observer};
    pub use crate::fms::*;
    pub use crate::layout::*;
    pub use crate::lottie_renderer::*;
    pub use crate::markers::*;
    pub use crate::state_machine::events::*;
    pub use crate::state_machine::listeners::*;
    pub use crate::state_machine::parser::*;
    pub use crate::state_machine::states::*;
    pub use crate::state_machine::transitions::guard::*;
    pub use crate::state_machine::transitions::*;
    pub use crate::state_machine::*;
    pub use crate::theming::*;
}
