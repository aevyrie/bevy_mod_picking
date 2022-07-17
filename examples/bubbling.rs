use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::{
    output::{EventData, EventListener, PointerClick, PointerOver},
    DebugEventsPlugin, DefaultPickingPlugins, PickRaycastSource, PickRaycastTarget, PickableBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_plugin(DebugEventsPlugin) // <- Adds debug event logging.
        .insert_resource(PresentMode::Mailbox)
        .add_startup_system(setup)
        .run();
}

struct MyEvent;

fn delete_target(commands: &mut Commands, event_data: &mut EventData<PointerClick>) {
    commands.entity(event_data.target()).despawn_recursive();
    event_data.event();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    let parent = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_scale(Vec3::new(1.0, 0.1, 1.0)),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
        .insert(PickRaycastTarget::default()) // <- Needed for the raycast backend.
        .insert(EventListener::<PointerClick>::run_commands(delete_target))
        .id();

    let children: Vec<Entity> = (0..100)
        .map(|i| {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_xyz(2.0, i as f32 * 0.5 - 25.0, 0.0),
                    ..Default::default()
                })
                .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
                .insert(PickRaycastTarget::default()) // <- Needed for the raycast backend.
                .id()
        })
        .collect();

    commands.entity(parent).push_children(&children);

    // light
    commands.spawn_bundle(PointLightBundle {
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
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.
}
