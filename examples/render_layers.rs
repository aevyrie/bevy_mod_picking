//! Demonstrates that picking respects render layers and order.

use bevy::{render::camera::ClearColorConfig, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_render::view::RenderLayers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
        ))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));
    // sphere
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    subdivisions: 3,
                    radius: 0.5,
                })
                .unwrap(),
            ),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        RenderLayers::layer(1),
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera transform
    let camera_transform = Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    // camera
    commands.spawn((Camera3dBundle {
        transform: camera_transform,
        ..default()
    },));
    // camera 2
    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            camera: Camera {
                clear_color: ClearColorConfig::None,
                // renders after / on top of the main camera
                order: 1,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
    ));
}
