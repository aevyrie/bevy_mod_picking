use bevy::prelude::*;
use bevy_mod_picking::{
    output::{EventData, EventFrom, EventListenerCommands, PointerClick},
    DebugEventsPlugin, DefaultPickingPlugins, PickRaycastSource, PickRaycastTarget, PickableBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_plugin(DebugEventsPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .add_system(DeleteMe::handle_events)
        .run();
}

struct DeleteMe(Entity);
impl EventFrom<PointerClick> for DeleteMe {
    fn new(event_data: &mut EventData<PointerClick>) -> Self {
        Self(event_data.target())
    }
}
impl DeleteMe {
    fn handle_events(mut commands: Commands, mut events: EventReader<DeleteMe>) {
        for event in events.iter() {
            commands.entity(event.0).despawn();
            info!("I deleted the thing!");
        }
    }
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
        .insert_bundle(PickableBundle::default())
        .insert(PickRaycastTarget::default())
        .forward_events::<PointerClick, DeleteMe>()
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
                .insert_bundle(PickableBundle::default())
                .insert(PickRaycastTarget::default())
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
