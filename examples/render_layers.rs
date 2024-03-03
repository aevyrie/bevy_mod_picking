//! Demonstrates that picking respects render layers and order.

use bevy::{prelude::*, render::camera::ClearColorConfig};
use bevy_mod_picking::prelude::*;
use bevy_render::view::RenderLayers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
        ))
        .insert_resource(DebugPickingMode::Normal)
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
            mesh: meshes.add(bevy_render::mesh::PlaneMeshBuilder {
                half_size: Vec2::splat(2.5),
                ..default()
            }),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));
    // sphere
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::default()),
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
