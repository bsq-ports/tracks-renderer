use bevy::prelude::*;
use bevy::prelude::shape;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, mpsc::Receiver};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InputCommand {
    Key { key: String, pressed: bool },
    MouseMove { dx: f32, dy: f32 },
    MouseButton { button: String, pressed: bool },
    Scroll { delta: f32 },
}

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

#[derive(Component)]
struct Rotating;

#[derive(Default, Resource)]
struct MouseState {
    left: bool,
    middle: bool,
    right: bool,
}

#[derive(Resource)]
struct SharedReceiver(Arc<Mutex<Receiver<InputCommand>>>);

pub fn start_bevy(rx: Receiver<InputCommand>) {
    let shared_rx = Arc::new(Mutex::new(rx));

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SharedReceiver(shared_rx))
        .insert_resource(MouseState::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_input_system,
                update_camera_transform_system,
                animate_cube_system,
            ),
        )
        .run();
}

// Web/WASM startup: when compiled for wasm (with `wasm` feature), use this
// function to start a Bevy app that renders to the web backend.
#[cfg(feature = "wasm")]
pub async fn start_bevy_wasm(_canvas_id: &str) {
    console_error_panic_hook::set_once();
    use bevy::prelude::*;

    // Build and run the Bevy app for web.
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(WebGL2Plugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_input_system,
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
    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.12, 0.12),
            ..default()
        }),
        ..default()
    });

    // Cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Rotating,
    ));

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera with orbit component
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera {
            target: Vec3::ZERO,
            yaw: 0.0,
            pitch: -0.3,
            distance: 6.0,
        },
    ));
}

fn process_input_system(
    rx: Res<SharedReceiver>,
    mut mouse: ResMut<MouseState>,
    mut cam_query: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
) {
    while let Ok(cmd) = rx.0.lock().unwrap().try_recv() {
        match cmd {
            InputCommand::Key { .. } => {
                // reserved for future use
            }
            InputCommand::MouseButton { button, pressed } => match button.as_str() {
                "Left" => mouse.left = pressed,
                "Middle" => mouse.middle = pressed,
                "Right" => mouse.right = pressed,
                _ => {}
            },
            InputCommand::MouseMove { dx, dy } => {
                for (_transform, mut orbit) in cam_query.iter_mut() {
                    if mouse.right {
                        orbit.yaw -= dx * 0.01;
                        orbit.pitch += dy * 0.01;
                        orbit.pitch = orbit.pitch.clamp(-1.5, 1.5);
                    }
                    if mouse.middle {
                        let right = Vec3::new(orbit.yaw.cos(), 0.0, orbit.yaw.sin());
                        orbit.target += -right * dx * 0.05 + Vec3::Y * (-dy * 0.05);
                    }
                }
            }
            InputCommand::Scroll { delta } => {
                for (_transform, mut orbit) in cam_query.iter_mut() {
                    orbit.distance = (orbit.distance - delta * 0.2).clamp(0.5, 100.0);
                }
            }
        }
    }
}

fn update_camera_transform_system(mut query: Query<(&mut Transform, &OrbitCamera), With<Camera3d>>) {
    for (mut transform, orbit) in query.iter_mut() {
        let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.distance * orbit.pitch.sin();
        let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();
        let pos = Vec3::new(x, y, z) + orbit.target;
        *transform = Transform::from_translation(pos).looking_at(orbit.target, Vec3::Y);
    }
}

fn animate_cube_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotating>>) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.8)
            * Quat::from_rotation_x(time.elapsed_secs() * 0.4);
    }
}
