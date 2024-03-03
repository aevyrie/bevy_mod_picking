//! Demonstrates how to change debug settings

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(low_latency_window_plugin())
                .set(bevy::log::LogPlugin {
                    filter: "bevy_mod_picking=trace".into(), // Show picking logs trace level and up
                    ..default()
                }),
        )
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        // Set the value of the DebugPickingMode resource to change picking debugging settings
        .insert_resource(DebugPickingMode::Normal)
        // A system that cycles the debugging state when you press F3:
        .add_systems(
            PreUpdate,
            (|mut mode: ResMut<DebugPickingMode>| {
                *mode = match *mode {
                    DebugPickingMode::Disabled => DebugPickingMode::Normal,
                    DebugPickingMode::Normal => DebugPickingMode::Noisy,
                    DebugPickingMode::Noisy => DebugPickingMode::Disabled,
                }
            })
            .distributive_run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F3,
            )),
        )
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
