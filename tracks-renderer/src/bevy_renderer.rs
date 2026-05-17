use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
// Note: We no longer need serde or std::sync channels for input!

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

#[derive(Component)]
struct Rotating;

/// Desktop/Native Entry Point
pub fn start_bevy() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_native_input_system,
                update_camera_transform_system,
                animate_cube_system,
            ),
        )
        .run();
}

/// WebAssembly Entry Point
#[cfg(feature = "wasm")]
pub async fn start_bevy_wasm(canvas_selector: &str) {
    let mut app = App::new();

    // Configure Bevy to hook directly into your specific HTML5 Canvas element
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some(canvas_selector.to_string()),
            // Optional: Prevents the browser window from scrolling when using the wheel over the canvas
            prevent_default_event_handling: true,
            ..default()
        }),
        ..default()
    }));

    app.add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_native_input_system,
                update_camera_transform_system,
                animate_cube_system,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Infinite Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100000.0, 100000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.12, 0.12),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::IDENTITY,
    ));

    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::ONE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Rotating,
    ));

    // Light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));

    // Camera with an extended Far Clipping Plane for infinite grounds
    commands.spawn((
        Camera3d { ..default() },
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            target: Vec3::ZERO + Vec3::Y * 0.5, // Look slightly above the ground plane
            yaw: 0.0,
            pitch: -0.3,
            distance: 6.0,
        },
    ));
}

/// **Native Input System**: Leverages Bevy's built-in accumulated mouse resources
/// for clean, aggregate, frame-rate independent viewing adjustments.
fn process_native_input_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>, // Built-in keyboard manager
    mouse_buttons: Res<ButtonInput<MouseButton>>, // Built-in mouse button manager
    mouse_motion: Res<AccumulatedMouseMotion>, // NEW: Automatically accumulates continuous frame motion
    mouse_scroll: Res<AccumulatedMouseScroll>, // NEW: Automatically accumulates continuous scroll updates
    mut cam_query: Query<&mut OrbitCamera, With<Camera3d>>,
) {
    // 1. Blender Style: Handle Right-Click dragging to rotate view matrix orientation
    if mouse_buttons.pressed(MouseButton::Right) {
        // Grab the pre-accumulated delta directly from the resource (no manual loop required)
        let delta = mouse_motion.delta;

        if delta.x != 0.0 || delta.y != 0.0 {
            for mut orbit in cam_query.iter_mut() {
                orbit.yaw -= delta.x * 0.005;
                orbit.pitch -= delta.y * 0.005;
                orbit.pitch = orbit.pitch.clamp(-1.4, 1.4);
            }
        }
    }

    // 2. Proportional Zoom: Handle Scroll Wheel input updates
    // Grab the pre-accumulated continuous wheel tracking variable
    let scroll_delta = mouse_scroll.delta.y;

    if scroll_delta != 0.0 {
        // 1. Apply a dampening multiplier (e.g., 0.05) to bring browser pixel deltas
        // into a manageable range, while keeping desktop line scrolling functional.
        let structural_delta = scroll_delta * 0.05;

        for mut orbit in cam_query.iter_mut() {
            // 2. Scale the zoom factor exponentially based on current distance.
            // This ensures zooming is granular when close up, and swift when far away.
            let zoom_factor = orbit.distance * 0.1;

            // 3. Compute and clamp the target distance safely
            orbit.distance = (orbit.distance - structural_delta * zoom_factor).clamp(1.5, 100.0);
        }
    }

    // 3. Spectator Panning: Process real-time keyboard inputs
    for mut orbit in cam_query.iter_mut() {
        let move_speed = 6.0 * time.delta_secs();

        let forward = Vec3::new(orbit.yaw.sin(), 0.0, orbit.yaw.cos()).normalize_or_zero();
        let right = Vec3::new(orbit.yaw.cos(), 0.0, -orbit.yaw.sin()).normalize_or_zero();

        if keys.pressed(KeyCode::KeyW) {
            orbit.target -= forward * move_speed;
        }
        if keys.pressed(KeyCode::KeyS) {
            orbit.target += forward * move_speed;
        }
        if keys.pressed(KeyCode::KeyA) {
            orbit.target -= right * move_speed;
        }
        if keys.pressed(KeyCode::KeyD) {
            orbit.target += right * move_speed;
        }
        if keys.pressed(KeyCode::Space) {
            orbit.target += Vec3::Y * move_speed;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            orbit.target -= Vec3::Y * move_speed;
        }
        // rotate right
        if keys.pressed(KeyCode::KeyQ) {
            orbit.yaw -= move_speed * 0.5;
        }
        if keys.pressed(KeyCode::KeyE) {
            orbit.yaw += move_speed * 0.5;
        }
    }
}

fn update_camera_transform_system(
    mut query: Query<(&mut Transform, &OrbitCamera), With<Camera3d>>,
) {
    for (mut transform, orbit) in query.iter_mut() {
        let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.distance * orbit.pitch.sin();
        let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();

        let camera_pos = Vec3::new(x, y, z) + orbit.target;

        transform.translation = camera_pos;
        transform.look_at(orbit.target, Vec3::Y);
    }
}

fn animate_cube_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotating>>) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.8)
            * Quat::from_rotation_x(time.elapsed_secs() * 0.4);
    }
}
