use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, Window, WindowOptions};
use rapier2d::prelude::*;
use std::time::Instant;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    sq_left_handle: RigidBodyHandle,
    sq_right_handle: RigidBodyHandle,
}

impl PhysicsWorld {
    fn new() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let gravity = vector![0.0, -9.81];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();

        // Create floor rectangle (fixed)
        let floor_body = RigidBodyBuilder::fixed()
            .translation(vector![0.0, -2.35])
            .build();
        let floor_handle = rigid_body_set.insert(floor_body);
        let floor_collider = ColliderBuilder::cuboid(0.6, 1.48).build();
        collider_set.insert_with_parent(floor_collider, floor_handle, &mut rigid_body_set);

        // Create left square (dynamic)
        let sq_left_body = RigidBodyBuilder::dynamic()
            .translation(vector![-0.885, 3.0])
            .build();
        let sq_left_handle = rigid_body_set.insert(sq_left_body);
        let sq_left_collider = ColliderBuilder::cuboid(0.621, 0.621).build();
        collider_set.insert_with_parent(sq_left_collider, sq_left_handle, &mut rigid_body_set);

        // Create right square (dynamic)
        let sq_right_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.784, 3.0])
            .build();
        let sq_right_handle = rigid_body_set.insert(sq_right_body);
        let sq_right_collider = ColliderBuilder::cuboid(0.621, 0.621).build();
        collider_set.insert_with_parent(sq_right_collider, sq_right_handle, &mut rigid_body_set);

        Self {
            rigid_body_set,
            collider_set,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            sq_left_handle,
            sq_right_handle,
        }
    }

    fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
    }

    fn get_positions(&self) -> ((f32, f32), (f32, f32)) {
        let sq_left = &self.rigid_body_set[self.sq_left_handle];
        let sq_right = &self.rigid_body_set[self.sq_right_handle];
        let pos_left = sq_left.translation();
        let pos_right = sq_right.translation();

        ((pos_left.x, pos_left.y), (pos_right.x, pos_right.y))
    }
}

struct Player {
    player: DotLottiePlayer,
    last_update: Instant,
}

impl Player {
    fn new(animation_path: &str) -> Self {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            loop_animation: false,
            background_color: 0xffffffff,
            ..Default::default()
        });

        let is_dotlottie = animation_path.ends_with(".lottie");

        if is_dotlottie {
            let data = std::fs::read(animation_path).unwrap();
            player.load_dotlottie_data(&data, WIDTH as u32, HEIGHT as u32);
        } else {
            player.load_animation_path(animation_path, WIDTH as u32, HEIGHT as u32);
        }

        Self {
            player,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self) -> bool {
        let updated = self.player.tick();
        self.last_update = Instant::now();
        updated
    }

    fn frame_buffer(&self) -> &[u32] {
        let (ptr, len) = (self.player.buffer_ptr(), self.player.buffer_len());
        unsafe { std::slice::from_raw_parts(ptr as *const u32, len as usize) }
    }
}

fn main() {
    let mut window = Window::new(
        "Lottie Player with Physics (ESC to exit)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new("./src/bin/physics/falling_squares.lottie");
    let mut physics = PhysicsWorld::new();

    let binding_file_path = format!("./src/bin/physics/{}.json", "binding");
    let binding_file_data = std::fs::read_to_string(&binding_file_path).expect(&format!(
        "Failed to read binding file: {}",
        binding_file_path
    ));

    let parse = player.player.bindings_load_data(&binding_file_data);

    let theme = player.player.set_theme("squares_theme");

    println!("Parse succeeded: {}", parse);
    println!("Theme succeeded: {}", theme);

    let mut mx = 0.0;
    let mut my = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Step physics simulation
        physics.step();

        // Get current physics positions
        let ((left_x, left_y), (right_x, right_y)) = physics.get_positions();

        // Convert physics coordinates to Lottie coordinates (100x100 space)
        // physics_to_lottie: x = physics_x * 20 + 50, y = 50 - physics_y * 20
        let lottie_left_x = left_x * 20.0 + 50.0;
        let lottie_left_y = 50.0 - left_y * 20.0;
        let lottie_right_x = right_x * 20.0 + 50.0;
        let lottie_right_y = 50.0 - right_y * 20.0;

        // Update Lottie animation with physics positions
        player
            .player
            .mutate_vector_binding("sq_l", &[lottie_left_x.into(), lottie_left_y.into()]);
        player
            .player
            .mutate_vector_binding("sq_r", &[lottie_right_x.into(), lottie_right_y.into()]);

        // Enable mouse tracking
        // let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Discard);
        // mouse_pos.map(|mouse| {
        //     if mouse.0 != mx || mouse.1 != my {
        //         mx = mouse.0;
        //         my = mouse.1;
        //     }
        // });
        // if mx != 0.0 && my != 0.0 {
        //     player
        //         .player
        //         .mutate_vector_binding("sq_l", &[mx.into(), my.into()]);
        // }

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
