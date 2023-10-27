//! Demonstrates how to use the rapier picking backend.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            // We want to disable the raycast backend because it is enabled by default. All supplied
            // backends are optional. In your app, you can disable the default features of the
            // plugin and only enable the backends you want to use. Picking will still work if both
            // backends are enabled, but that would mean we wouldn't be able to test the rapier
            // backend in isolation.
            DefaultPickingPlugins.build().disable::<RaycastBackend>(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(RapierBackendSettings {
            require_markers: true, // Optional: only needed when you want fine-grained control over which cameras and entities should be used with the rapier picking backend. This is disabled by default, and no marker components are required on cameras or colliders. This resource is inserted by default, you only need to add it if you want to override the default settings.
        })
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
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::WHITE.into()),
            ..default()
        },
        Collider::cuboid(2.5, 0.01, 2.5),
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
        RapierPickable, // Optional: only required if `RapierBackendSettings::require_markers`
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Collider::cuboid(0.5, 0.5, 0.5),
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
        RapierPickable, // Optional: only required if `RapierBackendSettings::require_markers`
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RapierPickable, // Optional: only required if `RapierBackendSettings::require_markers`
    ));
}
