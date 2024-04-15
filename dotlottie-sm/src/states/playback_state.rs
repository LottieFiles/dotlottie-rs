use dotlottie_player_core::{Config, DotLottiePlayer};

use crate::{state::State, transition::Transition};

pub struct PlaybackState {
    config: Config,

    reset_context: bool,
    animation_id: String,
    width: u32,
    height: u32,
    transitions: Vec<Box<dyn Transition>>,
    // entry_actions: Vec<Box<dyn Transition>>,
    // exit_actions: Vec<Box<dyn Transition>>,
}

/**
 * PlaybackState is a state that is used to set the configuration of the player
 *
 * Notes:
 *
 * Currently width & height are passed due to having animation_id.
 * If there is an animation_id, we need to load a new animation, this requires width & height.
 */
impl PlaybackState {
    pub fn new(
        config: Config,
        reset_context: bool,
        animation_id: String,
        width: u32,
        height: u32,
        transitions: Vec<Box<dyn Transition>>,
    ) -> Self {
        Self {
            config,
            reset_context,
            animation_id,
            width,
            height,
            transitions,
        }
    }
}

impl State for PlaybackState {
    fn execute(&mut self, player: &mut DotLottiePlayer) {
        let config = self.config.clone();

        if self.animation_id != "" {
            player.load_animation(&self.animation_id, self.width, self.height);
        }

        player.set_config(config);
    }

    fn reset_context(&self) -> bool {
        self.reset_context
    }

    fn get_animation_id(&self) -> &String {
        &self.animation_id
    }

    fn get_transitions(&self) -> &Vec<Box<dyn Transition>> {
        &self.transitions
    }

    fn add_transition(&mut self, transition: Box<dyn Transition>) {
        self.transitions.push(transition);
    }

    fn remove_transition(&mut self, transition: Box<dyn Transition>) {
        todo!()
    }

    fn set_reset_context(&mut self, reset_context: bool) {
        self.reset_context = reset_context;
    }

    fn get_state_name(&self) -> &str {
        "PlaybackState"
    }
}
