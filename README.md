# Bevy-Earth
"We belong to the Earth," replied Captain Nemo, "and I am its enemy!"

Earth Simulation with Bevy
Overview
This project is a 3D Earth simulation using the Bevy game engine. It (will) includes features to visualize the Earth both above and below sea level, with full mouse integration for interaction and control. The goal is to simulate military assets and their movements around the globe.

Features
3D Earth model rendering
Mouse-controlled camera for navigation
Simulation of military assets
Use of Bevy game engine and Rust programming language
Getting Started
Prerequisites
Rust programming language: Install Rust
Bevy game engine: Add Bevy as a dependency in your Cargo.toml
Installation
Clone the repository:

git clone https://github.com/yourusername/earth-simulation.git
cd earth-simulation
Update Cargo.toml to include Bevy:

[dependencies]
bevy = "0.14"
bevy_input = "0.14"
bevy_mod_picking = "0.8"
Add the following assets to the assets directory:

albedo_ver2.png
bump.png
clouds.png
Running the Simulation
Navigate to the project directory:

cd earth-simulation
Run the simulation:

cargo run
Usage
Controls
Mouse Movement: Rotate the camera around the Earth
Mouse Wheel: Zoom in and out
Code Structure
Main Function
The entry point of the application, setting up the Bevy app and adding the necessary systems.

rust
Kopiera kod
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control_system)
        .run();
}
Setup System
Sets up the initial scene with a camera and a directional light.

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 800.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    generate_faces(&mut commands, &mut meshes, &mut materials, &asset_server);
}
Face Generation
Generates the faces of the Earth using a cube map technique, allowing for detailed texturing and normal mapping.

fn generate_faces(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let faces = vec![
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::NEG_Z,
    ];

    let offsets = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)];

    for direction in faces {
        for offset in &offsets {
            commands.spawn(PbrBundle {
                mesh: meshes.add(generate_face(direction, 100, offset.0, offset.1)),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("albedo_ver2.png")),
                    metallic_roughness_texture: Some(asset_server.load("bump.png")),
                    perceptual_roughness: 1.0,
                    normal_map_texture: Some(asset_server.load("clouds.png")),
                    ..default()
                }),
                ..default()
            });
        }
    }
}
Camera Control System
Handles the camera movement and zoom based on mouse input.

fn camera_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let mut rotation_delta = Vec2::ZERO;
    let mut zoom_delta = 0.0f32;

    for event in mouse_motion_events.read() {
        rotation_delta += event.delta;
    }

    for event in mouse_wheel_events.read() {
        zoom_delta += event.y;
    }

    if let Ok(mut transform) = query.get_single_mut() {
        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, -rotation_delta.x * 0.005);
        let pitch_rotation = Quat::from_axis_angle(transform.rotation * Vec3::X, -rotation_delta.y * 0.005);
        transform.rotation = yaw_rotation * pitch_rotation * transform.rotation;
        let forward = transform.rotation * Vec3::Z;
        transform.translation += forward * zoom_delta * 10.0;
    }
}
Contributing
Contributions are welcome! Please open an issue or submit a pull request for any changes or improvements.

//Carlo Jacal

License
This project is licensed under the MIT License.


