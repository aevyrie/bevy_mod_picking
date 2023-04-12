use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

/// This example is identical to the minimal example, except a cube has been added, that when
/// clicked on, won't deselect everything else you have selected.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
    ));

    // cube with NoDeselect
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(1.5, 0.5, 0.0),
                ..Default::default()
            },
            PickableBundle::default(),
            PickRaycastTarget::default(), // <- Needed for the raycast backend.
            NoDeselect, // <- When this entity is clicked, other entities won't be deselected.
        ))
        .remove::<PickSelection>(); // <- Removing this removes the entity's ability to be selected.

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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PickRaycastCamera::default(),
    ));
}
