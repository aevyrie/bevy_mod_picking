//! An example of picking a hidden mesh in your bevy app.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_picking_raycast::{bevy_mod_raycast::prelude::RaycastVisibility, RaycastBackendSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .insert_resource(RaycastBackendSettings {
            raycast_visibility: RaycastVisibility::Ignore, // Allows us to pick a hidden mesh
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
            visibility: Visibility::Hidden,
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
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
