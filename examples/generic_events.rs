//! This example is similar to the `event_listener` example, except we will demonstrate a more
//! advanced method of forwarding events that are generic.
//!
//! This allows us to use the same custom event for multiple pointer events, as usual, but with the
//! key difference that we can have different behavior depending on the pointer event that triggered
//! our event.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_startup_system(setup)
        .add_event::<SpecificEvent>()
        .add_event::<GeneralEvent>()
        .add_system(SpecificEvent::print_events)
        .add_system(GeneralEvent::print_events)
        .run();
}

// Here we are going to make our event generic over pointer events, so we can specify what to do for
// specific events.
//
// Why is this useful? It allows us to have different behaviors depending on what event triggered
// our custom event. In this example, we say "hello" when the pointer enters, and "goodbye" when it
// leaves, but while using the same generic event, instead of two different events.
struct SpecificEvent {
    entity: Entity,
    greeting: String,
}
// Here we are implementing event forwarding only for the `PointerOver` version of our event.
impl From<ListenedEvent<Over>> for SpecificEvent {
    fn from(event: ListenedEvent<Over>) -> Self {
        SpecificEvent {
            entity: event.target,
            greeting: "Hello".into(),
        }
    }
}
// Here we are implementing event forwarding only for `PointerOut` version of our event.
impl From<ListenedEvent<Out>> for SpecificEvent {
    fn from(event: ListenedEvent<Out>) -> Self {
        SpecificEvent {
            entity: event.target,
            greeting: "Goodbye".into(),
        }
    }
}
// Finally, do something with our events.
impl SpecificEvent {
    fn print_events(mut greet: EventReader<SpecificEvent>) {
        for event in greet.iter() {
            info!("Specific: {} {:?}!", event.greeting, event.entity);
        }
    }
}

// If you don't care what pointer event is triggering your event, and you want to have the same
// behavior in all cases, you can simply ignore the event type.
struct GeneralEvent;
impl<E: IsPointerEvent> From<ListenedEvent<E>> for GeneralEvent {
    fn from(_event: ListenedEvent<E>) -> GeneralEvent {
        GeneralEvent
    }
}
impl GeneralEvent {
    fn print_events(mut greet: EventReader<GeneralEvent>) {
        for _event in greet.iter() {
            info!("General: Some event was triggered, but we don't know which.");
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
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            },
            PickableBundle::default(),
            RaycastPickTarget::default(),
            OnPointer::<Over>::send_event::<SpecificEvent>(),
            OnPointer::<Out>::send_event::<SpecificEvent>(),
            OnPointer::<Down>::send_event::<GeneralEvent>(),
        ))
        // Because event forwarding can rely on event bubbling, events that target children of the
        // parent cube will also bubble up to this parent level and will fire off an event:
        .with_children(|parent| {
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..Default::default()
                },
                PickableBundle::default(),
                RaycastPickTarget::default(),
            ));
        });

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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        RaycastPickCamera::default(),
    )); // <- Sets the camera to use for picking.
}