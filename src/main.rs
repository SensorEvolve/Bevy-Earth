use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::mesh::{Mesh, Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::PI;

const EARTH_RADIUS: f32 = 300.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control_system)
        .run();
}

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
                    base_color_texture: Some(
                        asset_server.load("albedo_ver2.png"),
                    ),
                    metallic_roughness_texture: Some(
                        asset_server.load("bump.png"),
                    ),
                    perceptual_roughness: 1.0,
                    normal_map_texture: Some(
                        asset_server.load("clouds.png"),
                    ),
                    ..default()
                }),
                ..default()
            });
        }
    }
}

pub fn generate_face(normal: Vec3, resolution: u32, x_offset: f32, y_offset: f32) -> Mesh {
    let axis_a = Vec3::new(normal.y, normal.z, normal.x); // Horizontal
    let axis_b = axis_a.cross(normal); // Vertical

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for y in 0..resolution {
        for x in 0..resolution {
            let i = x + y * resolution;

            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point_on_unit_cube = normal + (percent.x - x_offset) * axis_a + (percent.y - y_offset) * axis_b;
            let normalized_point = point_on_unit_cube.normalize() * EARTH_RADIUS;

            let coords: Coordinates = point_on_unit_cube.normalize().into();
            let (u, v) = coords.convert_to_uv_mercator();

            vertices.push(normalized_point.into());
            normals.push((-point_on_unit_cube.normalize()).into());
            uvs.push([u, v]);

            if x != resolution - 1 && y != resolution - 1 {
                // First triangle
                indices.push(i as u32);
                indices.push((i + resolution) as u32);
                indices.push((i + resolution + 1) as u32);

                // Second triangle
                indices.push(i as u32);
                indices.push((i + resolution + 1) as u32);
                indices.push((i + 1) as u32);
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.generate_tangents().unwrap();
    mesh
}

#[derive(Debug)]
pub struct Coordinates {
    // Stored internally in radians
    pub latitude: f32,
    pub longitude: f32,
}

impl From<Vec3> for Coordinates {
    fn from(value: Vec3) -> Self {
        let normalized_point = value.normalize();
        let latitude = normalized_point.y.asin();
        let longitude = normalized_point.x.atan2(normalized_point.z);
        Coordinates {
            latitude,
            longitude,
        }
    }
}

impl Coordinates {
    pub fn as_degrees(&self) -> (f32, f32) {
        let latitude = self.latitude * (180.0 / PI);
        let longitude = self.longitude * (180.0 / PI);
        (latitude, longitude)
    }

    pub fn convert_to_uv_mercator(&self) -> (f32, f32) {
        let (lat, lon) = self.as_degrees();
        let v = map_latitude(lat).unwrap();
        let u = map_longitude(lon).unwrap();
        (u, v)
    }

    #[allow(dead_code)]
    pub fn from_degrees(latitude: f32, longitude: f32) -> Result<Self, CoordError> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(CoordError {
                msg: format!("Invalid latitude: {}", latitude),
            });
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(CoordError {
                msg: format!("Invalid longitude: {}", longitude),
            });
        }
        let latitude = latitude / (180.0 / PI);
        let longitude = longitude / (180.0 / PI);
        Ok(Coordinates {
            latitude,
            longitude,
        })
    }

    pub fn get_point_on_sphere(&self) -> Vec3 {
        let y = self.latitude.sin();
        let r = self.latitude.cos();
        let x = self.longitude.sin() * -r;
        let z = self.longitude.cos() * r;
        Vec3::new(x, y, z).normalize() * EARTH_RADIUS
    }
}

fn map_latitude(lat: f32) -> Result<f32, CoordError> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(CoordError {
            msg: format!("Invalid latitude: {}", lat),
        });
    }
    if (90.0..=0.0).contains(&lat) {
        Ok(map((90.0, 0.0), (0.0, 0.5), lat))
    } else {
        Ok(map((0.0, -90.0), (0.5, 1.0), lat))
    }
}

fn map_longitude(lon: f32) -> Result<f32, CoordError> {
    if !(-180.0..=180.0).contains(&lon) {
        return Err(CoordError {
            msg: format!("Invalid longitude: {}", lon),
        });
    }
    if (-180.0..=0.0).contains(&lon) {
        Ok(map((-180.0, 0.0), (0.0, 0.5), lon))
    } else {
        Ok(map((0.0, 180.0), (0.5, 1.0), lon))
    }
}

fn map(input_range: (f32, f32), output_range: (f32, f32), value: f32) -> f32 {
    output_range.0 + (value - input_range.0) * (output_range.1 - output_range.0) / (input_range.1 - input_range.0)
}

#[derive(Debug)]
pub struct CoordError {
    msg: String,
}

fn camera_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let mut rotation_delta = Vec2::ZERO;
    let mut zoom_delta = 0.0f32; // Correct type to f32

    // Process mouse motion events
    for event in mouse_motion_events.read() {
        rotation_delta += event.delta;
    }

    // Process mouse wheel events
    for event in mouse_wheel_events.read() {
        zoom_delta += event.y;
    }

    // Apply the transformations to the camera
    if let Ok(mut transform) = query.get_single_mut() {
        // Rotate the camera around the Y axis (horizontal movement)
        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, -rotation_delta.x * 0.005);
        // Rotate the camera around the local X axis (vertical movement)
        let pitch_rotation = Quat::from_axis_angle(transform.rotation * Vec3::X, -rotation_delta.y * 0.005);

        transform.rotation = yaw_rotation * pitch_rotation * transform.rotation;

        // Calculate the forward direction based on the current rotation
        let forward = transform.rotation * Vec3::Z;

        // Move the camera forward/backward based on zoom
        transform.translation += forward * zoom_delta * 10.0;
    }
}

