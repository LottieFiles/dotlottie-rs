#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{state_machine::OpenUrlPolicy, ColorSpace, Event, NodeProps, Player};
    use std::ffi::CString;

    const W: u32 = 300;
    const H: u32 = 220;

    fn setup(buffer: &mut Vec<u32>) -> Player {
        let mut player = Player::new();
        assert!(player
            .set_sw_target(buffer, W, H, ColorSpace::ABGR8888)
            .is_ok());
        let data = CString::new(include_str!("../../motion-assets/hover-card.json")).unwrap();
        assert!(player.load_animation_data(&data).is_ok());
        player
    }

    fn tick_for(sm: &mut dotlottie_rs::StateMachineEngine, ms: f32) {
        let mut elapsed = 0.0;
        while elapsed < ms {
            let _ = sm.tick(16.0);
            elapsed += 16.0;
        }
    }

    #[test]
    fn set_node_entry_action_applies_override() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": [],
                 "entryActions": [
                    {"type": "SetNode", "target": "icon",
                     "props": {"rotate": 30, "opacity": 0.5}}
                 ]}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        let props = sm.player.get_node_props("icon").expect("override present");
        assert_eq!(props.rotate, Some(30.0));
        assert_eq!(props.opacity, Some(0.5));
    }

    #[test]
    fn animate_action_binds_inputs_and_fires_on_complete() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "",
                 "transitions": [
                    {"type": "Transition", "toState": "b",
                     "guards": [{"type": "Event", "inputName": "done"}]}
                 ],
                 "entryActions": [
                    {"type": "Animate", "target": "icon",
                     "keyframes": {"x": "$amt"},
                     "transition": {"duration": 0.2, "ease": "linear"},
                     "onComplete": "done"}
                 ]},
                {"type": "PlaybackState", "name": "b", "animation": "", "transitions": []}
            ],
            "inputs": [
                {"type": "Numeric", "name": "amt", "value": 40},
                {"type": "Event", "name": "done"}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));
        assert_eq!(sm.get_current_state_name(), "a");

        tick_for(&mut sm, 400.0);

        assert_eq!(sm.get_current_state_name(), "b");
        let props = sm.player.get_node_props("icon").expect("override present");
        assert!(
            (props.x.unwrap() - 40.0).abs() < 0.5,
            "x settled at {:?}",
            props.x
        );
    }

    #[test]
    fn motion_sequences_mixed_steps_and_fires_on_motion_complete() {
        let sm_def = r#"{
            "initial": "a",
            "motions": [
                {"name": "seq", "steps": [
                    {"target": "icon", "keyframes": {"y": 10},
                     "transition": {"duration": 0.1, "ease": "linear"}},
                    {"type": "SetNode", "target": "icon", "props": {"opacity": 0.5}},
                    {"target": "icon", "keyframes": {"y": 0},
                     "transition": {"duration": 0.1, "ease": "linear"}}
                ]}
            ],
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "",
                 "motions": ["seq"], "transitions": []}
            ],
            "interactions": [
                {"type": "OnMotionComplete", "motionName": "seq",
                 "actions": [{"type": "SetBoolean", "inputName": "flag", "value": true}]}
            ],
            "inputs": [{"type": "Boolean", "name": "flag", "value": false}]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        // Mid-sequence: the instant SetNode between the two animates has not run yet.
        tick_for(&mut sm, 48.0);
        let props = sm.player.get_node_props("icon").expect("override");
        assert_eq!(props.opacity, None);

        tick_for(&mut sm, 400.0);
        let props = sm.player.get_node_props("icon").expect("override");
        assert_eq!(props.opacity, Some(0.5), "SetNode step ran between animates");
        assert!(props.y.unwrap().abs() < 0.5, "settled home, y {:?}", props.y);
        assert_eq!(sm.get_boolean_input("flag"), Some(true));
    }

    #[test]
    fn at_offsets_join_one_batch() {
        let sm_def = r#"{
            "initial": "a",
            "motions": [
                {"name": "stagger", "steps": [
                    {"target": "icon", "keyframes": {"x": 20},
                     "transition": {"duration": 0.05, "ease": "linear"}, "at": 0},
                    {"target": "card", "keyframes": {"x": 20},
                     "transition": {"duration": 0.05, "ease": "linear"}, "at": "+0.05"}
                ]}
            ],
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "",
                 "motions": ["stagger"], "transitions": []}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        tick_for(&mut sm, 400.0);
        let icon = sm.player.get_node_props("icon").expect("icon");
        let card = sm.player.get_node_props("card").expect("card");
        assert!((icon.x.unwrap() - 20.0).abs() < 0.5);
        assert!((card.x.unwrap() - 20.0).abs() < 0.5, "staggered step ran");
    }

    #[test]
    fn infinite_motion_is_interrupted_on_state_exit() {
        let sm_def = r#"{
            "initial": "a",
            "motions": [
                {"name": "pulse", "repeat": "infinite", "steps": [
                    {"target": "icon", "keyframes": {"y": 10},
                     "transition": {"duration": 0.05, "ease": "linear"}},
                    {"target": "icon", "keyframes": {"y": 0},
                     "transition": {"duration": 0.05, "ease": "linear"}}
                ]}
            ],
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "",
                 "motions": ["pulse"],
                 "transitions": [
                    {"type": "Transition", "toState": "b",
                     "guards": [{"type": "Event", "inputName": "go"}]}
                 ]},
                {"type": "PlaybackState", "name": "b", "animation": "", "transitions": []}
            ],
            "inputs": [{"type": "Event", "name": "go"}]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        tick_for(&mut sm, 130.0);
        assert!(
            sm.player.get_node_props("icon").is_some(),
            "pulse is writing overrides"
        );

        assert!(sm.fire("go", true).is_ok());
        assert_eq!(sm.get_current_state_name(), "b");

        let frozen = sm.player.get_node_props("icon").unwrap().y;
        tick_for(&mut sm, 130.0);
        let after = sm.player.get_node_props("icon").unwrap().y;
        assert_eq!(frozen, after, "state-scoped motion froze on exit");
    }

    #[test]
    fn on_input_change_runs_actions() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": []}
            ],
            "interactions": [
                {"type": "OnInputChange", "inputName": "progress",
                 "actions": [
                    {"type": "SetNode", "target": "icon", "props": {"x": "$progress"}}
                 ]}
            ],
            "inputs": [{"type": "Numeric", "name": "progress", "value": 0}]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        sm.set_numeric_input("progress", 42.0, true, false);
        let props = sm.player.get_node_props("icon").expect("override");
        assert_eq!(props.x, Some(42.0));

        sm.set_numeric_input("progress", 7.0, true, false);
        assert_eq!(sm.player.get_node_props("icon").unwrap().x, Some(7.0));
    }

    #[test]
    fn animate_input_drives_a_numeric_input() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": [],
                 "entryActions": [
                    {"type": "AnimateInput", "inputName": "counter",
                     "from": 0, "to": 100,
                     "transition": {"duration": 0.2, "ease": "linear"}}
                 ]}
            ],
            "inputs": [{"type": "Numeric", "name": "counter", "value": 0}]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        tick_for(&mut sm, 96.0);
        let mid = sm.get_numeric_input("counter").unwrap();
        assert!(mid > 0.0 && mid < 100.0, "mid-flight value {mid}");

        tick_for(&mut sm, 300.0);
        assert_eq!(sm.get_numeric_input("counter"), Some(100.0));
    }

    #[test]
    fn unknown_action_types_are_skipped_not_fatal() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": [],
                 "entryActions": [
                    {"type": "SomeFutureAction", "target": "icon"},
                    {"type": "SetNode", "target": "icon", "props": {"rotate": 15}}
                 ]}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("must parse");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));
        assert_eq!(sm.player.get_node_props("icon").unwrap().rotate, Some(15.0));
    }

    #[test]
    fn pointer_reserved_inputs_resolve() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": []}
            ],
            "interactions": [
                {"type": "PointerDown",
                 "actions": [
                    {"type": "SetNode", "target": "icon",
                     "props": {"x": "@pointer.x", "y": "@pointer.y"}}
                 ]}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        sm.post_event(&Event::PointerDown { x: 33.0, y: 7.0 });
        let props = sm.player.get_node_props("icon").expect("override");
        assert_eq!(props.x, Some(33.0));
        assert_eq!(props.y, Some(7.0));
    }

    #[test]
    fn sm_stop_reverts_sm_written_nodes_only() {
        let sm_def = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": [],
                 "entryActions": [
                    {"type": "SetNode", "target": "icon", "props": {"rotate": 30}}
                 ]}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        // Host-set override on a node the SM never touched.
        let host_props = NodeProps {
            x: Some(5.0),
            ..Default::default()
        };
        assert!(sm.player.set_node_props("card", host_props).is_ok());

        assert!(sm.player.get_node_props("icon").is_some());
        sm.stop();

        assert!(
            sm.player
                .get_node_props("icon")
                .map(|p| p.rotate.is_none())
                .unwrap_or(true),
            "SM-written override reverted on stop"
        );
        assert_eq!(
            sm.player.get_node_props("card").and_then(|p| p.x),
            Some(5.0),
            "host override untouched"
        );
    }

    #[test]
    fn play_motion_action_and_ignore_interrupt_policy() {
        let sm_def = r#"{
            "initial": "a",
            "motions": [
                {"name": "drop", "interrupt": "ignore", "steps": [
                    {"target": "icon", "keyframes": {"y": [0, 50]},
                     "transition": {"duration": 0.1, "ease": "linear"}}
                ]}
            ],
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": []}
            ],
            "interactions": [
                {"type": "PointerDown",
                 "actions": [{"type": "PlayMotion", "motion": "drop"}]}
            ]
        }"#;
        let mut buffer = vec![0u32; (W * H) as usize];
        let mut player = setup(&mut buffer);
        let mut sm = player.state_machine_load_data(sm_def).expect("load");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        sm.post_event(&Event::PointerDown { x: 1.0, y: 1.0 });
        tick_for(&mut sm, 48.0);
        let mid = sm.player.get_node_props("icon").unwrap().y.unwrap();
        assert!(mid > 0.0 && mid < 50.0, "mid-flight y {mid}");

        // Re-trigger while running: "ignore" keeps the live instance (no restart
        // to the explicit 0 start).
        sm.post_event(&Event::PointerDown { x: 1.0, y: 1.0 });
        let _ = sm.tick(16.0);
        let after = sm.player.get_node_props("icon").unwrap().y.unwrap();
        assert!(after >= mid, "no restart: {after} >= {mid}");

        tick_for(&mut sm, 200.0);
        assert!((sm.player.get_node_props("icon").unwrap().y.unwrap() - 50.0).abs() < 0.5);
    }
}
