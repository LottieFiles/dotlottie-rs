use crate::{state::State, transition::Transition};

pub struct SyncState {
    frame_context_key: String,
    reset_context: bool,
    animation_id: String,
    width: u32,
    height: u32,
    transitions: Vec<Box<dyn Transition>>,
}

/**
* SyncState is a state that is used to sync the player with the frame
* value contained inside the StateMachine's context.
*
* Notes:
* How can we grab the StateMachine and request a context variable?
*/
impl SyncState {
    pub fn new(
        frame_context_key: String,
        reset_context: bool,
        animation_id: String,
        width: u32,
        height: u32,
        transitions: Vec<Box<dyn Transition>>,
    ) -> Self {
        Self {
            frame_context_key,
            reset_context,
            animation_id,
            width,
            height,
            transitions,
        }
    }
}

impl State for SyncState {
    fn execute(&mut self, player: &mut dotlottie_player_core::DotLottiePlayer) {
        if self.animation_id != "" {
            player.load_animation(&self.animation_id, self.width, self.height);
        }

        // some how grab the StateMachine and request a context variable
        println!(">> SyncState: execute {0}", self.frame_context_key)
    }

    fn reset_context(&self) -> bool {
        self.reset_context
    }

    fn get_animation_id(&self) -> &String {
        &self.animation_id
    }

    fn get_transitions(&self) -> &Vec<Box<dyn crate::transition::Transition>> {
        &self.transitions
    }

    fn add_transition(&mut self, transition: Box<dyn crate::transition::Transition>) {
        self.transitions.push(transition);
    }

    fn remove_transition(&mut self, transition: Box<dyn crate::transition::Transition>) {
        todo!()
    }

    fn set_reset_context(&mut self, reset_context: bool) {
        self.reset_context = reset_context;
    }

    fn get_state_name(&self) -> &str {
        "SyncState"
    }
}
