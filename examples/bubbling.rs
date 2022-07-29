//! This example demonstrates how event bubbling can be used to propagate events up an entity
//! hierarchy, as well as how event listeners can be used to forward events for specific entities -
//! triggered when a specific pointer event occurs.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(backends::RaycastPlugin)
        .add_plugin(DebugEventsPlugin)
        .add_startup_system(setup)
        .add_system(handle_events)
        .run();
}

struct DeleteMe(Entity);
impl EventFrom for DeleteMe {
    fn new(event_data: &mut EventData<impl IsPointerEvent>) -> Self {
        // Note that we forward the target, not the entity! The target is the child that the event
        // was originally called on, whereas the listener is the parent entity that was listening
        // for the event that bubbled up from the target.
        Self(event_data.target())
    }
}

struct GreetMe(Entity);
impl EventFrom for GreetMe {
    fn new(event_data: &mut EventData<impl IsPointerEvent>) -> Self {
        Self(event_data.target())
    }
}

fn handle_events(
    mut commands: Commands,
    mut delete: EventReader<DeleteMe>,
    mut greet: EventReader<GreetMe>,
) {
    for event in delete.iter() {
        commands.entity(event.0).despawn_recursive();
        info!("I deleted the thing!");
    }
    for event in greet.iter() {
        info!("Hello {:?}!", event.0);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(PickRaycastTarget::default())
        // Check out this neat trick!
        //
        // Because event forwarding can rely on event bubbling, events that target children of the
        // scene will bubble up to this level and will fire off a `GreetMe` or `DeleteMe` event,
        // depending on the event that bubbled up:
        .forward_events::<PointerClick, DeleteMe>()
        .forward_events::<PointerOver, GreetMe>()
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..Default::default()
                })
                .insert_bundle(PickableBundle::default())
                .insert(PickRaycastTarget::default());
        });

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
