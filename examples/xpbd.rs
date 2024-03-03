//! Demonstrates how to use the xpbd picking backend.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_xpbd_3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            // We want to disable the raycast backend because it is enabled by default. All supplied
            // backends are optional. In your app, you can disable the default features of the
            // plugin and only enable the backends you want to use. Picking will still work if both
            // backends are enabled, but that would mean we wouldn't be able to test the xpbd
            // backend in isolation.
            DefaultPickingPlugins.build().disable::<RaycastBackend>(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(DebugPickingMode::Normal)
        .insert_resource(XpbdBackendSettings {
            require_markers: true, // Optional: only needed when you want fine-grained control over which cameras and entities should be used with the xpbd picking backend. This is disabled by default, and no marker components are required on cameras or colliders. This resource is inserted by default, you only need to add it if you want to override the default settings.
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
            mesh: meshes.add(bevy_render::mesh::PlaneMeshBuilder {
                half_size: Vec2::splat(2.5),
                ..default()
            }),
            material: materials.add(Color::WHITE),
            ..default()
        },
        Collider::cuboid(5.0, 0.01, 5.0),
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
        XpbdPickable, // Optional: only required if `XpbdBackendSettings::require_markers`
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Collider::cuboid(1.0, 1.0, 1.0),
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
        XpbdPickable, // Optional: only required if `XpbdBackendSettings::require_markers`
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
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
        XpbdPickable, // Optional: only required if `XpbdBackendSettings::require_markers`
    ));
}
