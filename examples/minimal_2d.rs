use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins::build(RaycastBackend))
        .add_plugin(DebugEventsPlugin::default())
        .add_startup_system(setup)
        .run();
}

/// Set up a simple 2D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::splat(128.)),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        })
        .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
        .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.
}
