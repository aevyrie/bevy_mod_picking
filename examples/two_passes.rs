//! Demonstrates that picking respects camera render order.

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_mod_picking::prelude::*;

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
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
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
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
    ));

    // plane 2
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::CYAN.into()),
            transform: Transform::from_xyz(20., 20., 20.),
            ..default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));
    // cube 2
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::YELLOW.into()),
            transform: Transform::from_xyz(20., 20.5, 20.),
            ..default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(18.0, 22.0, 28.0),
        ..default()
    });
    // camera 2
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(30., 30., 30.0)
                .looking_at(Vec3::new(20., 20.5, 20.), Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera: Camera {
                // renders after / on top of the main camera
                order: 1,
                ..default()
            },
            ..default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
    ));
}
