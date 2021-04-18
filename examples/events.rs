use bevy::prelude::*;
use bevy_mod_picking::{
    HighlightablePickingPlugin, InteractablePickingPlugin, PickableBundle, PickingCameraBundle,
    PickingEvent, PickingPlugin,
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        // PickingPlugin provides core picking systems and must be registered first
        .add_plugin(PickingPlugin)
        // InteractablePickingPlugin adds mouse events and selection
        .add_plugin(InteractablePickingPlugin)
        // HighlightablePickingPlugin adds hover, click, and selection highlighting
        .add_plugin(HighlightablePickingPlugin)
        .add_startup_system(setup.system())
        .add_system_to_stage(CoreStage::PostUpdate, print_events.system())
        .run();
}

pub fn print_events(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        info!("{:?}", event);
    }
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
        .insert_bundle(PickableBundle::default());

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default());
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
}
