use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::{HighlightablePickingPlugins, DefaultPickingPlugins, NoDeselect, PickableBundle, PickingCameraBundle};

/// This example is identical to the 3d_scene example, except a cube has been added, that when
/// clicked on, won't deselect everything else you have selected.
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
        .insert(PickableBundle::default());

    // cube
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.0, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(PickableBundle::default());
    // cube with NoDeselect
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.5, 0.5, 0.0),
            ..Default::default()
        })
        .insert(PickableBundle::default())
        .insert(NoDeselect);
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PickingCameraBundle::default());
}
