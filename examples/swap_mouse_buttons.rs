//! The minimal example, but with the right mouse button as primary,
//! the left button as secondary, and the middle button not mapped.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use input::InputPluginSettings;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        // All you need to do is add the picking plugin, with your backend of choice enabled in the
        // cargo features. By default, the bevy_mod_raycast backend is enabled via the
        // `backend_raycast` feature.
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .run();
}

// Spawn a simple scene, like bevy's 3d_scene example.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut input_plugin_settings: ResMut<InputPluginSettings>,
) {
    input_plugin_settings
        .mouse_button_mapping
        .set_mapping(MouseButton::Left, Some(PointerButton::Secondary));
    input_plugin_settings
        .mouse_button_mapping
        .set_mapping(MouseButton::Right, Some(PointerButton::Primary));
    input_plugin_settings
        .mouse_button_mapping
        .set_mapping(MouseButton::Middle, None);

    // The rest of this is identical to minimal.rs
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        PickableBundle::default(), // Adds selection, highlighting, and the `Pickable` override.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        PickableBundle::default(), // Adds selection, highlighting, and the `Pickable` override.
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
