use dotlottie_player_core::states::StateTrait;
use dotlottie_player_core::transitions::TransitionTrait;
use dotlottie_player_core::DotLottiePlayer;
pub const WIDTH: u32 = 100;
pub const HEIGHT: u32 = 100;

// Helper function to get the current transition event as a string
pub fn get_current_transition_event(player: &DotLottiePlayer) -> String {
    let players_first_state = player
        .get_state_machine()
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_current_state()
        .unwrap();

    let players_first_transition =
        &*players_first_state.read().unwrap().get_transitions()[0].clone();

    let complete_event = &*players_first_transition.read().unwrap();

    let complete_event_string = complete_event.get_event().read().unwrap().as_str().clone();

    return complete_event_string.clone();
}
