//! This example is similar to the `bubbling` example, except we will demonstrate a more advanced
//! method of forwarding events that are generic.
//!
//! This allows us to use the same event for multiple pointer events, as usual, but with the key
//! difference that we can have different behavior depending on the pointer event that triggered our
//! event.

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(RaycastPlugin)
        .add_plugin(DebugEventsPlugin)
        .add_startup_system(setup)
        .add_system(GreetMe::<PointerOver>::handle_events)
        .add_system(GreetMe::<PointerOut>::handle_events)
        .run();
}

// Here we are going to make our event generic over pointer events, so we can specify what to do for
// specific events.
//
// Why is this useful? It allows us to have different behaviors depending on what event triggered
// our custom event. In this example, we say "hello" when the pointer enters, and "goodbye" when it
// leaves, but while using the same generic event, instead of two different events.
struct GreetMe<E: IsPointerEvent> {
    entity: Entity,
    greeting: String,
    event: PhantomData<E>,
}
// Here we are implementing event forwarding only for the `PointerOver` version of our event.
impl ForwardedEvent for GreetMe<PointerOver> {
    fn from_data<E: IsPointerEvent>(event_data: &PointerEventData<E>) -> GreetMe<PointerOver> {
        GreetMe {
            entity: event_data.target(),
            greeting: "Hello".into(),
            event: PhantomData,
        }
    }
}
// Here we are implementing event forwarding only for `PointerOut` version of our event.
impl ForwardedEvent for GreetMe<PointerOut> {
    fn from_data<E: IsPointerEvent>(event_data: &PointerEventData<E>) -> GreetMe<PointerOut> {
        GreetMe {
            entity: event_data.target(),
            greeting: "Goodbye".into(),
            event: PhantomData,
        }
    }
}
// Finally, this is our event handler that prints out the greetings. Note this is a generic system,
// so we need to add both the `PointerOver` and `PointerOut` versions of our system. The advantage
// to this is that we can define our greeting logic once, but it can handle multiple types of
// events.
impl<E: IsPointerEvent> GreetMe<E> {
    fn handle_events(mut greet: EventReader<GreetMe<E>>) {
        for event in greet.iter() {
            info!("{} {:?}!", event.greeting, event.entity);
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
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(PickRaycastTarget::default())
        // Because event forwarding can rely on event bubbling, events that target children of the
        // parent cube will bubble up to this level and will fire off a `GreetMe` event:
        .forward_events::<PointerOver, GreetMe<PointerOver>>()
        .forward_events::<PointerOut, GreetMe<PointerOut>>()
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
