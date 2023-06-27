//! Demonstrates how to use the rapier picking backend.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_picking_rapier::RapierPickTarget;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
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
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        Collider::cuboid(2.5, 0.01, 2.5),
        PickableBundle::default(),   // <- Makes the collider pickable.
        RapierPickTarget::default(), // <- Needed for the rapier picking backend
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        Collider::cuboid(0.5, 0.5, 0.5),
        PickableBundle::default(),   // <- Makes the collider pickable.
        RapierPickTarget::default(), // <- Needed for the rapier picking backend
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        RapierPickCamera::default(), // <- Sets the camera to use for picking.
    ));
}
