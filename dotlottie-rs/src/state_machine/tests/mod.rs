// mod dotlottie_sm;
#[test]
fn parse_state_machine_test() {
    use crate::DotLottieManager;
    use crate::{StateActionJson, StateJson, StringEventJson, TransitionJson};

    use std::{fs::File, io::Read};

    let file_path = format!(
        "{}{}",
        env!("CARGO_MANIFEST_DIR"),
        "/src/tests/resources/pigeon_fsm.json"
    );

    let mut sm_definition = File::open(file_path).unwrap();
    let mut buffer_to_string = String::new();

    sm_definition.read_to_string(&mut buffer_to_string).unwrap();

    let dotlottie = DotLottieManager::new(None).unwrap();

    let state_machine_json = dotlottie.parse_state_machine(&buffer_to_string).unwrap();

    assert_eq!(state_machine_json.states.len(), 3);
    assert_eq!(state_machine_json.transitions.len(), 3);

    let start_action = StateActionJson {
        r#type: "LogAction".to_string(),
        target: None,
        url: None,
        theme_id: None,
        sound_id: None,
        message: Some("Howdy partner!".to_string()),
    };

    let entry_action_vec = vec![start_action];

    let pigeon_state_0: StateJson = StateJson {
        r#type: "PlaybackState".to_string(),
        animation_id: None,
        r#loop: Some(true),
        autoplay: Some(true),
        mode: Some("forward".to_string()),
        speed: Some(1.0),
        marker: Some("bird".to_string()),
        segment: Some([].to_vec()),
        background_color: None,
        use_frame_interpolation: Some(true),
        entry_actions: Some(entry_action_vec),
        exit_actions: Some(vec![]),
        frame_context_key: None,
        reset_context: None,
    };

    let pigeon_state_1: StateJson = StateJson {
        r#type: "PlaybackState".to_string(),
        animation_id: None,
        r#loop: Some(false),
        autoplay: Some(true),
        mode: Some("forward".to_string()),
        speed: Some(0.5),
        marker: Some("explosion".to_string()),
        segment: Some([].to_vec()),
        background_color: None,
        use_frame_interpolation: Some(true),
        entry_actions: Some(vec![]),
        exit_actions: Some(vec![]),
        frame_context_key: None,
        reset_context: None,
    };

    let pigeon_state_2: StateJson = StateJson {
        r#type: "PlaybackState".to_string(),
        animation_id: None,
        r#loop: Some(false),
        autoplay: Some(true),
        mode: Some("forward".to_string()),
        speed: Some(1.0),
        marker: Some("feather".to_string()),
        segment: Some([].to_vec()),
        background_color: None,
        use_frame_interpolation: Some(true),
        entry_actions: Some(vec![]),
        exit_actions: Some(vec![]),
        frame_context_key: None,
        reset_context: None,
    };

    let pigeon_transition_0_string_event = StringEventJson {
        value: "explosion".to_string(),
    };
    let pigeon_transition_1_string_event = StringEventJson {
        value: "complete".to_string(),
    };
    let pigeon_transition_2_string_event = StringEventJson {
        value: "complete".to_string(),
    };
    let pigeon_transition_0 = TransitionJson {
        r#type: "Transition".to_string(),
        from_state: 0,
        to_state: 1,
        string_event: Some(pigeon_transition_0_string_event),
        guard: None,
        numeric_event: None,
        boolean_event: None,
        on_complete_event: None,
        on_pointer_down_event: None,
        on_pointer_up_event: None,
        on_pointer_enter_event: None,
        on_pointer_exit_event: None,
        on_pointer_move_event: None,
    };
    let pigeon_transition_1 = TransitionJson {
        r#type: "Transition".to_string(),
        from_state: 1,
        to_state: 2,
        string_event: Some(pigeon_transition_1_string_event),
        guard: None,
        numeric_event: None,
        boolean_event: None,
        on_complete_event: None,
        on_pointer_down_event: None,
        on_pointer_up_event: None,
        on_pointer_enter_event: None,
        on_pointer_exit_event: None,
        on_pointer_move_event: None,
    };
    let pigeon_transition_2 = TransitionJson {
        r#type: "Transition".to_string(),
        from_state: 2,
        to_state: 0,
        string_event: Some(pigeon_transition_2_string_event),
        guard: None,
        numeric_event: None,
        boolean_event: None,
        on_complete_event: None,
        on_pointer_down_event: None,
        on_pointer_up_event: None,
        on_pointer_enter_event: None,
        on_pointer_exit_event: None,
        on_pointer_move_event: None,
    };

    let states = vec![pigeon_state_0, pigeon_state_1, pigeon_state_2];
    let transitions = vec![
        pigeon_transition_0,
        pigeon_transition_1,
        pigeon_transition_2,
    ];
    let mut i = 0;

    for state in state_machine_json.states {
        assert_eq!(state == states[i], true);
        i += 1;
    }
    i = 0;
    for transition in state_machine_json.transitions {
        assert_eq!(transition == transitions[i], true);
        i += 1;
    }
}
