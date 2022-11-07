use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::{DebugEventsPickingPlugin, HighlightablePickingPlugins, DefaultPickingPlugins, PickableBundle, PickingCameraBundle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction plugins.
        .add_plugins(HighlightablePickingPlugins) // <- Adds Highlighting plugins.
        .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .run();
}

/// set up a simple 2D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::splat(128.)),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        })
        .insert(PickableBundle::default()); // <- Makes the mesh pickable.
                                                   // camera
    commands
        .spawn(Camera2dBundle::default())
        .insert(PickingCameraBundle::default()); // <- Sets the camera to use for picking.
}
