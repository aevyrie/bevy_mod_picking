use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::prelude::*;

/// This example is identical to the minimal example, except a cube has been added, that when
/// clicked on, won't deselect everything else you have selected.
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::AutoNoVsync, // Reduce input latency
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins::build(RaycastBackend))
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
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.

    // cube with NoDeselect
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.5, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .remove::<PickSelection>() // <- Removing this removes the entity's ability to be selected.
        .insert(PickRaycastTarget::default()) // <- Needed for the raycast backend.
        .insert(NoDeselect); // <- This ensures that when this entity is clicked, other entities won't be deselected.

    // light
    commands.spawn_bundle(PointLightBundle {
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
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PickRaycastSource::default());
}
