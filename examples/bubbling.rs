//! This example demonstrates how event bubbling can be used to propagate events up an entity
//! hierarchy, as well as how event listeners can be used to forward events to specific entities
//! when a specific pointer event occurs.
//!
//! The Big Idea here is to make it easy to couple interaction events with specific entities. In
//! other words, it allows you to easily implement "If entity X is hovered/clicked/dragged, do Y".
//!
//! The `forward_events` function might seem like magic, but it's pretty straightforward under the
//! hood. It simply adds an [`EventListener`](bevy_picking_core::output::EventListener) component to
//! the entity. When the event bubbling system encounters this `EventListener`, it uses the
//! [`ForwardedEvent`] trait you implemented on your custom event to convert the `PointerEvent` into
//! your custom event.
//!
//! In other words, the `forward_events` function is really a helper function that just inserts a
//! predefined `EventListener` component on your entity.

use bevy::prelude::*;
use bevy_mod_picking::prelude::{
    backends::raycast::{PickRaycastSource, PickRaycastTarget, RaycastBackend},
    *,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins::start().with_backend(RaycastBackend))
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .add_system(DeleteMe::handle_events)
        .add_system(GreetMe::handle_events)
        .run();
}

// We want to implement a feature that will allow us to click on a cube to delete it. To do this,
// we'll start by making an event we can send when we want to delete an entity.
struct DeleteMe(Entity);
// We're going to use the event forwarding feature of this crate to send a `DeleteMe` event when the
// entity is clicked. To be able to forward events, we need to implement the `ForwardedEvent` trait
// on our custom `DeleteMe` event.
//
// All we're doing is defining how to take a pointer event and turn it into our custom event.
impl ForwardedEvent<PointerClick> for DeleteMe {
    fn from_data(event_data: &PointerEventData<PointerClick>) -> DeleteMe {
        // Note that we are using the `target()` entity here, not the listener entity! The target is
        // the child that the event was originally called on, whereas the listener is the ancestor
        // that was listening for the event that bubbled up from the target.
        DeleteMe(event_data.target())
        // Why is this useful? It allows us to add an event listener once on the parent entity, yet
        // it can trigger actions specific to the child that was interacted with! Instead of needing
        // to add an event listener on every child, we can just stick one on the parent, and any
        // events that happen on the children will bubble up the the parent and be handled there.
    }
}
impl DeleteMe {
    // Here we will implement the system that does something with our `DeleteMe` events.
    fn handle_events(mut commands: Commands, mut delete: EventReader<DeleteMe>) {
        for event in delete.iter() {
            commands.entity(event.0).despawn_recursive();
            info!("I deleted the thing!");
        }
    }
}

// Same concept as the `DeleteMe` event, but just says "Hello!" to the entity.
struct GreetMe(Entity);
impl ForwardedEvent<PointerOver> for GreetMe {
    fn from_data(event_data: &PointerEventData<PointerOver>) -> GreetMe {
        GreetMe(event_data.target())
    }
}
impl GreetMe {
    fn handle_events(mut greet: EventReader<GreetMe>) {
        for event in greet.iter() {
            info!("Hello {:?}!", event.0);
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            },
            PickableBundle::default(),
            PickRaycastTarget::default(),
        ))
        // Because event forwarding uses bubbling, events that target children of the parent cube
        // will bubble up to this level and will fire off a `GreetMe` or `DeleteMe` event, depending
        // on the event that bubbled up:
        .forward_events::<PointerClick, DeleteMe>()
        .forward_events::<PointerOver, GreetMe>()
        .with_children(|parent| {
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..Default::default()
                },
                // As noted above, we are adding a child here but we don't need to add an
                // event listener. Events on this child will bubble up to the parent!
                PickableBundle::default(),
                PickRaycastTarget::default(),
            ));
        });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PickRaycastSource::default(),
    ));
}
