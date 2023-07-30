//! A minimal 2d example.

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .run();
}

/// Set up a simple 2D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default()
                .with_translation(Vec3::splat(128.))
                .with_scale(Vec3::splat(128.)),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));

    commands.spawn((Camera2dBundle::default(), RaycastPickCamera::default())); // <- Sets the camera to use for picking.
}
