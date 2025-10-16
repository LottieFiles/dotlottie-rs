use dotlottie_rs::{Config, DotLottiePlayer, LayerBoundingBox};
use minifb::{Key, Window, WindowOptions};
use rapier2d::prelude::*;
use std::time::Instant;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

// Coordinate conversion constants
const LOTTIE_WIDTH: f32 = 512.0;
const LOTTIE_HEIGHT: f32 = 512.0;
const LOTTIE_CENTER_X: f32 = LOTTIE_WIDTH / 2.0;
const LOTTIE_CENTER_Y: f32 = LOTTIE_HEIGHT / 2.0;
const PIXELS_PER_UNIT: f32 = 20.0; // Adjusted for 512x512 canvas

// Convert from Lottie coordinates to physics coordinates
fn lottie_to_physics(lottie_x: f32, lottie_y: f32) -> (f32, f32) {
    let physics_x = (lottie_x - LOTTIE_CENTER_X) / PIXELS_PER_UNIT;
    let physics_y = (LOTTIE_CENTER_Y - lottie_y) / PIXELS_PER_UNIT;
    (physics_x, physics_y)
}

// Convert from physics coordinates to Lottie coordinates
fn physics_to_lottie(physics_x: f32, physics_y: f32) -> (f32, f32) {
    let lottie_x = physics_x * PIXELS_PER_UNIT + LOTTIE_CENTER_X;
    let lottie_y = LOTTIE_CENTER_Y - physics_y * PIXELS_PER_UNIT;
    (lottie_x, lottie_y)
}

// Convert size from Lottie to physics (same for width/height)
fn lottie_size_to_physics(lottie_size: f32) -> f32 {
    lottie_size / PIXELS_PER_UNIT
}

// Extract center position and dimensions from bounding box
fn bbox_to_physics(bbox: &LayerBoundingBox) -> (f32, f32, f32, f32) {
    // Calculate center
    let lottie_x = (bbox.x1 + bbox.x2 + bbox.x3 + bbox.x4) / 4.0;
    let lottie_y = (bbox.y1 + bbox.y2 + bbox.y3 + bbox.y4) / 4.0;

    // Calculate width and height
    let width = (bbox.x2 - bbox.x1).abs();
    let height = (bbox.y3 - bbox.y1).abs();

    // Convert to physics
    let (physics_x, physics_y) = lottie_to_physics(lottie_x, lottie_y);
    let physics_half_width = lottie_size_to_physics(width) / 2.0;
    let physics_half_height = lottie_size_to_physics(height) / 2.0;

    (
        physics_x,
        physics_y,
        physics_half_width,
        physics_half_height,
    )
}

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
    ball_handle: RigidBodyHandle,
    sq_left_handle: RigidBodyHandle,
    sq_right_handle: RigidBodyHandle,
}

impl PhysicsWorld {
    fn new(
        floor_bbox: &LayerBoundingBox,
        sq_l_bbox: &LayerBoundingBox,
        sq_r_bbox: &LayerBoundingBox,
        ball_bbox: &LayerBoundingBox,
    ) -> Self {
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

        // Create floor rectangle (fixed) from bounding box
        let (floor_x, floor_y, floor_hw, floor_hh) = bbox_to_physics(floor_bbox);
        let floor_body = RigidBodyBuilder::fixed()
            .translation(vector![floor_x, floor_y])
            .build();
        let floor_handle = rigid_body_set.insert(floor_body);
        let floor_collider = ColliderBuilder::cuboid(floor_hw, floor_hh)
            .restitution(0.8) // Make floor bouncy too
            .build();
        collider_set.insert_with_parent(floor_collider, floor_handle, &mut rigid_body_set);

        // Create ball (dynamic) from bounding box
        let (ball_x, ball_y, ball_hw, _ball_hh) = bbox_to_physics(ball_bbox);
        let ball_radius = ball_hw; // Use half-width as radius for circle
        let ball_body = RigidBodyBuilder::dynamic()
            .translation(vector![ball_x, ball_y])
            .build();
        let ball_handle = rigid_body_set.insert(ball_body);
        let ball_collider = ColliderBuilder::ball(ball_radius)
            .restitution(0.9) // 0.9 = very bouncy (90% energy retained)
            .friction(0.3)
            .build();
        collider_set.insert_with_parent(ball_collider, ball_handle, &mut rigid_body_set);

        // Create left square (dynamic) from bounding box
        let (sq_left_x, sq_left_y, sq_left_hw, sq_left_hh) = bbox_to_physics(sq_l_bbox);
        let sq_left_body = RigidBodyBuilder::dynamic()
            .translation(vector![sq_left_x, sq_left_y])
            .build();
        let sq_left_handle = rigid_body_set.insert(sq_left_body);
        let sq_left_collider = ColliderBuilder::cuboid(sq_left_hw, sq_left_hh)
            .restitution(0.1) // Low bounce
            .friction(0.8) // High friction for tumbling
            .build();
        collider_set.insert_with_parent(sq_left_collider, sq_left_handle, &mut rigid_body_set);

        // Create right square (dynamic) from bounding box
        let (sq_right_x, sq_right_y, sq_right_hw, sq_right_hh) = bbox_to_physics(sq_r_bbox);
        let sq_right_body = RigidBodyBuilder::dynamic()
            .translation(vector![sq_right_x, sq_right_y])
            .build();
        let sq_right_handle = rigid_body_set.insert(sq_right_body);
        let sq_right_collider = ColliderBuilder::cuboid(sq_right_hw, sq_right_hh)
            .restitution(0.1) // Low bounce
            .friction(0.8) // High friction for tumbling
            .build();
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
            ball_handle,
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

    fn get_positions(&self) -> ((f32, f32), (f32, f32, f32), (f32, f32, f32)) {
        let ball = &self.rigid_body_set[self.ball_handle];
        let sq_left = &self.rigid_body_set[self.sq_left_handle];
        let sq_right = &self.rigid_body_set[self.sq_right_handle];

        let pos_ball = ball.translation();
        let pos_left = sq_left.translation();
        let pos_right = sq_right.translation();

        // Get rotation angles in radians
        let rot_left = sq_left.rotation().angle();
        let rot_right = sq_right.rotation().angle();

        (
            (pos_ball.x, pos_ball.y),
            (pos_left.x, pos_left.y, rot_left),
            (pos_right.x, pos_right.y, rot_right),
        )
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

    let binding_file_path = format!("./src/bin/physics/{}.json", "binding");
    let binding_file_data = std::fs::read_to_string(&binding_file_path).expect(&format!(
        "Failed to read binding file: {}",
        binding_file_path
    ));

    let parse = player.player.bindings_load_data(&binding_file_data);
    let theme = player.player.set_theme("squares_theme");

    println!("Parse succeeded: {}", parse);
    println!("Theme succeeded: {}", theme);

    // Get bounding boxes for all layers
    let floor_bbox = player.player.get_layer_bounds("floor");
    let sq_l_bbox = player.player.get_layer_bounds("sq_l");
    let sq_r_bbox = player.player.get_layer_bounds("sq_r");
    let ball_bbox = player.player.get_layer_bounds("ball");

    println!(
        "Floor bbox: ({}, {}) to ({}, {})",
        floor_bbox.x1, floor_bbox.y1, floor_bbox.x3, floor_bbox.y3
    );
    println!(
        "Sq_L bbox: ({}, {}) to ({}, {})",
        sq_l_bbox.x1, sq_l_bbox.y1, sq_l_bbox.x3, sq_l_bbox.y3
    );
    println!(
        "Sq_R bbox: ({}, {}) to ({}, {})",
        sq_r_bbox.x1, sq_r_bbox.y1, sq_r_bbox.x3, sq_r_bbox.y3
    );
    println!(
        "Ball bbox: ({}, {}) to ({}, {})",
        ball_bbox.x1, ball_bbox.y1, ball_bbox.x3, ball_bbox.y3
    );

    // Initialize physics with bounding boxes
    let mut physics = PhysicsWorld::new(&floor_bbox, &sq_l_bbox, &sq_r_bbox, &ball_bbox);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Step physics simulation
        physics.step();

        // Get current physics positions and rotations
        let ((ball_x, ball_y), (left_x, left_y, left_rot), (right_x, right_y, right_rot)) =
            physics.get_positions();

        // Debug: print positions occasionally
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 30 == 0 {
                println!(
                    "Ball: ({:.2}, {:.2}), Left: ({:.2}, {:.2}), Right: ({:.2}, {:.2})",
                    ball_x, ball_y, left_x, left_y, right_x, right_y
                );
            }
        }

        // Convert physics coordinates to Lottie coordinates
        let (lottie_ball_x, lottie_ball_y) = physics_to_lottie(ball_x, ball_y);
        let (lottie_left_x, lottie_left_y) = physics_to_lottie(left_x, left_y);
        let (lottie_right_x, lottie_right_y) = physics_to_lottie(right_x, right_y);

        // Convert rotation from radians to degrees (Lottie uses degrees)
        let left_rot_deg = left_rot.to_degrees();
        let right_rot_deg = right_rot.to_degrees();

        // Update Lottie animation with physics positions
        player
            .player
            .mutate_vector_binding("ball", &[lottie_ball_x.into(), lottie_ball_y.into()]);
        player
            .player
            .mutate_vector_binding("sq_l", &[lottie_left_x.into(), lottie_left_y.into()]);
        player
            .player
            .mutate_vector_binding("sq_r", &[lottie_right_x.into(), lottie_right_y.into()]);

        // Update rotations
        player
            .player
            .mutate_scalar_binding("sq_l_rot", left_rot_deg.into());
        player
            .player
            .mutate_scalar_binding("sq_r_rot", right_rot_deg.into());

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
