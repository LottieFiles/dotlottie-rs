mod test_utils;

use std::ffi::CString;

use crate::test_utils::{HEIGHT, WIDTH};
use dotlottie_rs::motion::{
    AnimateOptions, Prop, PropKeyframes, SpringParams, Transition, EASE_LINEAR,
};
use dotlottie_rs::{ColorSpace, NodeProps, Player, PlayerEvent};

fn loaded_player(buffer: &mut [u32]) -> Player {
    let mut player = Player::new();
    player
        .set_sw_target(buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    let path = CString::new("assets/animations/lottie/test.json").unwrap();
    player.load_animation_path(&path).unwrap();
    player
}

fn drain(player: &mut Player) -> Vec<PlayerEvent> {
    let mut events = Vec::new();
    while let Some(event) = player.poll_event() {
        events.push(event);
    }
    events
}

#[test]
fn set_and_get_node_props() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    player
        .set_node_props(
            "B",
            NodeProps {
                rotate: Some(45.0),
                opacity: Some(0.5),
                ..Default::default()
            },
        )
        .unwrap();

    let props = player.get_node_props("B").unwrap();
    assert_eq!(props.rotate, Some(45.0));
    assert_eq!(props.opacity, Some(0.5));

    // Overrides render even while the player is not playing.
    assert!(player.render().is_ok());

    player.reset_node("B").unwrap();
    assert!(player.render().is_ok());
    // The restore entry is dropped after one render.
    assert!(player.get_node_props("B").is_none());
}

#[test]
fn animate_ticks_to_completion_and_emits_event() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    let id = player.animate(
        "B",
        vec![PropKeyframes {
            prop: Prop::Rotate,
            values: vec![90.0],
            times: None,
        }],
        AnimateOptions {
            transition: Transition::Tween {
                duration: 0.2,
                easing: EASE_LINEAR,
            },
            delay: 0.0,
        },
    );

    for _ in 0..30 {
        let _ = player.tick(1000.0 / 60.0);
    }

    let events = drain(&mut player);
    assert!(
        events.contains(&PlayerEvent::MotionComplete { id }),
        "expected MotionComplete, got {events:?}"
    );

    let props = player.get_node_props("B").unwrap();
    assert_eq!(props.rotate, Some(90.0));
}

#[test]
fn spring_animation_settles_while_paused() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    let id = player.animate(
        "E",
        vec![PropKeyframes {
            prop: Prop::X,
            values: vec![120.0],
            times: None,
        }],
        AnimateOptions {
            transition: Transition::Spring(SpringParams::default()),
            delay: 0.0,
        },
    );

    // Not playing: the motion clock still runs on tick.
    for _ in 0..600 {
        let _ = player.tick(1000.0 / 60.0);
    }

    let events = drain(&mut player);
    assert!(events.contains(&PlayerEvent::MotionComplete { id }));
    assert_eq!(player.get_node_props("E").unwrap().x, Some(120.0));
}

#[test]
fn cancel_reverts_to_authored() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    let id = player.animate(
        "B",
        vec![PropKeyframes {
            prop: Prop::Rotate,
            values: vec![90.0],
            times: None,
        }],
        AnimateOptions::default(),
    );
    for _ in 0..5 {
        let _ = player.tick(1000.0 / 60.0);
    }
    assert!(player.get_node_props("B").unwrap().rotate.is_some());

    player.animation_cancel(id);
    // The cleared entry restores the authored pose on the next render, then drops.
    let _ = player.tick(1000.0 / 60.0);
    assert!(player.render().is_ok() || player.get_node_props("B").is_none());
    let _ = player.tick(1000.0 / 60.0);
    assert!(player.get_node_props("B").is_none());
}

#[test]
fn set_props_interrupts_running_track() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    player.animate(
        "B",
        vec![PropKeyframes {
            prop: Prop::Rotate,
            values: vec![90.0],
            times: None,
        }],
        AnimateOptions {
            transition: Transition::Tween {
                duration: 10.0,
                easing: EASE_LINEAR,
            },
            delay: 0.0,
        },
    );
    for _ in 0..5 {
        let _ = player.tick(1000.0 / 60.0);
    }

    player
        .set_node_props(
            "B",
            NodeProps {
                rotate: Some(10.0),
                ..Default::default()
            },
        )
        .unwrap();

    // The track is gone: further ticks must not overwrite the set value.
    for _ in 0..5 {
        let _ = player.tick(1000.0 / 60.0);
    }
    assert_eq!(player.get_node_props("B").unwrap().rotate, Some(10.0));
}

#[test]
fn stage_compositing_props_render() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    player
        .set_node_props(
            "@stage",
            NodeProps {
                tint: Some(dotlottie_rs::Tint {
                    black: [11, 16, 48],
                    white: [255, 217, 160],
                    intensity: 0.6,
                }),
                spot: Some(dotlottie_rs::SpotMask {
                    cx: 50.0,
                    cy: 50.0,
                    radius: 40.0,
                    feather: 0.5,
                }),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(player.render().is_ok());

    // Animate the spotlight; ticks must keep rendering.
    player.animate(
        "@stage",
        vec![PropKeyframes {
            prop: Prop::SpotRadius,
            values: vec![80.0],
            times: None,
        }],
        AnimateOptions::default(),
    );
    for _ in 0..30 {
        let _ = player.tick(1000.0 / 60.0);
    }
    assert_eq!(
        player
            .get_node_props("@stage")
            .unwrap()
            .spot
            .unwrap()
            .radius,
        80.0
    );

    // Clip circle via dotted-key scalars.
    player.animate(
        "@stage",
        vec![
            PropKeyframes {
                prop: Prop::ClipCx,
                values: vec![50.0],
                times: None,
            },
            PropKeyframes {
                prop: Prop::ClipCy,
                values: vec![50.0],
                times: None,
            },
            PropKeyframes {
                prop: Prop::ClipRadius,
                values: vec![45.0],
                times: None,
            },
        ],
        AnimateOptions::default(),
    );
    for _ in 0..30 {
        let _ = player.tick(1000.0 / 60.0);
    }
    assert!(matches!(
        player.get_node_props("@stage").unwrap().clip,
        Some(dotlottie_rs::ClipRegion::Circle { r, .. }) if (r - 45.0).abs() < 1e-3
    ));

    player.reset_node("@stage").unwrap();
    let _ = player.tick(1000.0 / 60.0);
    assert!(player.render().is_ok() || player.get_node_props("@stage").is_none());
}

#[test]
fn layer_blur_and_blend_apply() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    player
        .set_node_props(
            "E",
            NodeProps {
                blur: Some(6.0),
                blend_mode: Some(16), // Add
                ..Default::default()
            },
        )
        .unwrap();
    assert!(player.render().is_ok());

    player.animate(
        "E",
        vec![PropKeyframes {
            prop: Prop::Blur,
            values: vec![0.0],
            times: None,
        }],
        AnimateOptions::default(),
    );
    for _ in 0..30 {
        let _ = player.tick(1000.0 / 60.0);
    }
    assert_eq!(player.get_node_props("E").unwrap().blur, Some(0.0));
}

#[test]
fn duplicate_and_remove_node() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    player.duplicate_node("E", "ghost").unwrap();
    player
        .set_node_props(
            "ghost",
            NodeProps {
                opacity: Some(0.4),
                scale_x: Some(1.4),
                scale_y: Some(1.4),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(player.render().is_ok());

    // Duplicates animate like any node.
    let id = player.animate(
        "ghost",
        vec![PropKeyframes {
            prop: Prop::Opacity,
            values: vec![0.0],
            times: None,
        }],
        AnimateOptions::default(),
    );
    for _ in 0..40 {
        let _ = player.tick(1000.0 / 60.0);
    }
    let events = drain(&mut player);
    assert!(events.contains(&PlayerEvent::MotionComplete { id }));

    player.remove_node("ghost").unwrap();
    assert!(player.get_node_props("ghost").is_none());
    let _ = player.render();

    // Duplicate names must not collide with @stage or existing nodes.
    assert!(player.duplicate_node("E", "@stage").is_err());
}

#[test]
fn animate_value_ticks_and_reads() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = loaded_player(&mut buffer);

    let id = player.animate_value(
        0.0,
        100.0,
        AnimateOptions {
            transition: Transition::Tween {
                duration: 0.2,
                easing: EASE_LINEAR,
            },
            delay: 0.0,
        },
    );
    assert_eq!(player.animation_value(id), Some(0.0));
    for _ in 0..6 {
        let _ = player.tick(1000.0 / 60.0);
    }
    let mid = player.animation_value(id).unwrap();
    assert!(mid > 10.0 && mid < 90.0, "mid {mid}");
    for _ in 0..20 {
        let _ = player.tick(1000.0 / 60.0);
    }
    assert_eq!(player.animation_value(id), Some(100.0));
    let events = drain(&mut player);
    assert!(events.contains(&PlayerEvent::MotionComplete { id }));
}

#[test]
fn layers_lists_authored_names() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let player = loaded_player(&mut buffer);
    assert_eq!(player.layers(), ["R", "E", "B"]);
}

#[test]
fn layers_recurses_into_precomps() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let mut player = Player::new();
    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let data = r#"{"v":"5.7.4","fr":30,"ip":0,"op":30,"w":100,"h":100,
        "assets":[{"id":"pc1","layers":[
            {"ddd":0,"ind":1,"ty":4,"nm":"inner","sr":1,"ks":{},"ip":0,"op":30,"st":0,
             "shapes":[{"ty":"rc","p":{"a":0,"k":[50,50]},"s":{"a":0,"k":[40,40]},"r":{"a":0,"k":0}},
                       {"ty":"fl","c":{"a":0,"k":[1,0,0,1]},"o":{"a":0,"k":100}}]}]}],
        "layers":[
            {"ddd":0,"ind":1,"ty":0,"nm":"comp","refId":"pc1","sr":1,"ks":{},
             "w":100,"h":100,"ip":0,"op":30,"st":0},
            {"ddd":0,"ind":2,"ty":4,"nm":"front","sr":1,"ks":{},"ip":0,"op":30,"st":0,
             "shapes":[]}]}"#;
    let data = CString::new(data).unwrap();
    player.load_animation_data(&data).unwrap();

    assert_eq!(player.layers(), ["comp", "inner", "front"]);
}
