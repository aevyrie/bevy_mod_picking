use bevy::{prelude::*, window::PresentMode};

use bevy_mod_picking::{DebugCursorPickingPlugin, DebugEventsPickingPlugin, HighlightablePickingPlugins, DefaultPickingPlugins, PickableBundle, PickingCameraBundle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                present_mode: PresentMode::AutoNoVsync, // Reduce input latency
                ..default()
            },
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction plugins.
        .add_plugins(HighlightablePickingPlugins) // <- Adds Highlighting plugins.
        .add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
        .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert(PickableBundle::default()); // <- Makes the mesh pickable.
                                                   // cube
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(PickableBundle::default()); // <- Makes the mesh pickable.
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
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PickingCameraBundle::default()); // <- Sets the camera to use for picking.
}
