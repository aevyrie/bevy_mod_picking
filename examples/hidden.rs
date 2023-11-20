//! An example of picking a hidden mesh in your bevy app.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_picking_raycast::{bevy_mod_raycast::prelude::RaycastVisibility, RaycastBackendSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(RaycastBackendSettings {
            raycast_visibility: RaycastVisibility::Ignore,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, show)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
    ));
    commands.spawn((
        PbrBundle {
            visibility: Visibility::Hidden,
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        On::<Pointer<Over>>::target_component_mut::<Visibility>(|_listener, visibility| {
            *visibility = Visibility::Visible;
        }),
        On::<Pointer<Out>>::target_component_mut::<Visibility>(|_listener, visibility| {
            *visibility = Visibility::Hidden;
        }),
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
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

pub fn show(mut gizmos: Gizmos) {
    gizmos.cuboid(
        Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        Color::GREEN,
    );
}
