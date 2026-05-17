use bevy::{
    ecs::event::Trigger, input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll}, prelude::*
};
use std::{str::FromStr, sync::Mutex};

pub use tracks_rs::prelude::*;
use tracks_rs::{
    animation::coroutine_manager::CoroutineManager,
    point_definition::{
        Vector4PointDefinition, quaternion_point_definition::QuaternionPointDefinition,
        vector3_point_definition::Vector3PointDefinition,
    },
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// ============================================================================
// Resources & Structural Contexts
// ============================================================================

#[derive(Resource)]
pub struct BevyTracksContext {
    pub coroutine: Mutex<CoroutineManager>,
    pub base_provider: Mutex<BaseProviderContext>,
}

#[derive(Resource, Default)]
pub struct FrontendTime {
    pub seconds: f32,
}

// Global thread-safe hook allowing JS commands to talk directly to our Bevy App instance
static BEVY_APP_CHANNEL: Mutex<Option<App>> = Mutex::new(None);

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

#[derive(Component)]
struct Rotating;

// Markers containing pre-parsed tracking definitions
#[derive(Component)]
pub struct PositionTrack(pub Vector3PointDefinition);

#[derive(Component)]
pub struct RotationTrack(pub QuaternionPointDefinition);

#[derive(Component)]
pub struct ColorTrack(pub Vector4PointDefinition);

#[derive(Component)]
pub struct AnimationTrack {
    pub target_entity: Entity,
}

// ============================================================================
// App Entry Points
// ============================================================================

pub fn start_bevy() {
    let app = App::new().add_plugins(DefaultPlugins);

    configure_common_app(&mut app);

    app.run();
}

/// WebAssembly Entry Point
#[cfg(feature = "wasm")]
#[pub_async_if_wasm] // Custom helper or manual async depending on toolchain
pub async fn start_bevy_wasm(canvas_selector: &str) {
    let mut app = App::new();
    configure_common_app(&mut app);

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

    // Cache the app instance globally so the javascript runtime can drive it
    if let Ok(mut guard) = BEVY_APP_CHANNEL.lock() {
        *guard = Some(app);
    }
}

/// Registers the universal execution pipelines shared between native and browser runtimes
fn configure_common_app(app: &mut App) {
    // Instantiate core provider resources
    app.insert_resource(FrontendTime::default())
        .insert_resource(BevyTracksContext {
            coroutine: Mutex::new(CoroutineManager::default()),
            base_provider: Mutex::new(BaseProviderContext::default()),
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_native_input_system,
                update_camera_transform_system,
                animate_cube_system,
                process_position_animations_system,
                process_rotation_animations_system,
            ),
        );

    // Dynamic Observer: Automatically adds 3D primitives whenever the frontend requests a cube
    app.add_observer(
        |trigger: Trigger<OnAdd, Name>,
         mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            commands.entity(trigger.entity()).insert((
                Mesh3d(meshes.add(Cuboid::from_size(Vec3::ONE))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.6, 1.0),
                    ..default()
                })),
            ));
        },
    );
}

// ============================================================================
// Wasm-Bindgen Interface Boundaries (Frontend JS Commands)
// ============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn update_frontend_time(js_seconds: f32) {
    if let Ok(mut guard) = BEVY_APP_CHANNEL.lock() {
        if let Some(app) = guard.as_mut() {
            if let Some(mut time_res) = app.world_mut().get_resource_mut::<FrontendTime>() {
                time_res.seconds = js_seconds;
            }
            // Manually advance the engine update cycles alongside the frontend tick frame
            app.update();
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn spawn_cube_from_frontend(id_str: String, x: f32, y: f32, z: f32) {
    if let Ok(mut guard) = BEVY_APP_CHANNEL.lock() {
        if let Some(app) = guard.as_mut() {
            app.world_mut()
                .spawn((Name::new(id_str), Transform::from_xyz(x, y, z)));
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn add_json_animation_track(target_id: String, anim_type: String, json_data_str: String) {
    let anim_type = AnimationTrackType::from_str(&anim_type).unwrap_or_else(|_| {
        panic!("Invalid animation track type provided from frontend: {}", anim_type);
    });

    if let Ok(mut guard) = BEVY_APP_CHANNEL.lock() {
        if let Some(app) = guard.as_mut() {
            let world = app.world_mut();

            let mut system_state =
                bevy::ecs::system::SystemState::<Query<(Entity, &Name)>>::new(world);
            let query = system_state.get(world);
            let target_entity = query
                .iter()
                .find(|(_, name)| name.as_str() == target_id)
                .map(|(entity, _)| entity);

            if let Some(entity) = target_entity {
                // Parse once immediately upon arrival
                let parsed_json: serde_json::Value =
                    serde_json::from_str(&json_data_str).unwrap_or_default();

                let context = world.resource::<BevyTracksContext>();
                let mut provider_ctx = context.base_provider.lock().unwrap();
                let mut track_commands = world.spawn(AnimationTrack {
                    target_entity: entity,
                });

                match anim_type {
                    AnimationTrackType::Position => {
                        let parsed_def =
                            Vector3PointDefinition::parse(parsed_json, &mut provider_ctx);
                        track_commands.insert(PositionTrack(parsed_def));
                    }
                    AnimationTrackType::Rotation => {
                        let parsed_def =
                            QuaternionPointDefinition::parse(parsed_json, &mut provider_ctx);
                        track_commands.insert(RotationTrack(parsed_def));
                    }
                    AnimationTrackType::Color => {
                        let parsed_def =
                            Vector4PointDefinition::parse(parsed_json, &mut provider_ctx);
                        track_commands.insert(ColorTrack(parsed_def));
                    }
                    _ => {
                        error!("Unknown animation track type requested: {}", anim_type);
                    }
                }
            }
        }
    }
}

// ============================================================================
// Pipeline Core Systems
// ============================================================================

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
        Name::new("sandbox_center_cube"),
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
            target: Vec3::ZERO + Vec3::Y * 0.5,
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

fn process_position_animations_system(
    frontend_time: Res<FrontendTime>,
    tracks_query: Query<(&AnimationTrack, &PositionTrack)>,
    mut targets_query: Query<&mut Transform>,
) {
    let current_time = frontend_time.seconds;
    for (track, position_data) in tracks_query.iter() {
        if let Ok(mut transform) = targets_query.get_mut(track.target_entity) {
            transform.translation += position_data.0.evaluate(current_time);
        }
    }
}

fn process_rotation_animations_system(
    frontend_time: Res<FrontendTime>,
    tracks_query: Query<(&AnimationTrack, &RotationTrack)>,
    mut targets_query: Query<&mut Transform>,
) {
    let current_time = frontend_time.seconds;
    for (track, rotation_data) in tracks_query.iter() {
        if let Ok(mut transform) = targets_query.get_mut(track.target_entity) {
            transform.rotation = transform.rotation * rotation_data.0.evaluate(current_time);
        }
    }
}
