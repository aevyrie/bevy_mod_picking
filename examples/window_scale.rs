//! Tests window scaling.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy_window::PresentMode::AutoNoVsync,
                resolution:
                    bevy_window::WindowResolution::default().with_scale_factor_override(3.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(bevy_render::mesh::PlaneMeshBuilder {
                half_size: Vec2::splat(2.5),
                ..default()
            }),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..default()
    });
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
}
