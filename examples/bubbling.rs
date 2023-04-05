//! This example demonstrates how event bubbling can be used to propagate events up an entity
//! hierarchy, as well as how event listeners can be used to forward events to specific entities
//! when a specific pointer event occurs.
//!
//! The Big Idea here is to make it easy to couple interaction events with specific entities. In
//! other words, it allows you to easily implement "If entity X is hovered/clicked/dragged, do Y".
//!
//! The `forward_events` function might seem like magic, but it's pretty straightforward under the
//! hood. It simply adds an [`EventListener`](bevy_picking_core::events::EventListener) component to
//! the entity. When the event bubbling system encounters this `EventListener`, it uses the
//! [`ForwardedEvent`] trait you implemented on your custom event to convert the `PointerEvent` into
//! your custom event.
//!
//! In other words, the `forward_events` function is really a helper function that just inserts a
//! predefined `EventListener` component on your entity.

use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Bubble, EventListener},
    prelude::{
        backends::raycast::{PickRaycastSource, PickRaycastTarget},
        *,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .add_system(Greeting::handle_events)
        .run();
}

/// A callback function used with an `EventListener`.
fn delete_me(commands: &mut Commands, event: &EventData<PointerClick>, _: &mut Bubble) {
    // We don't want to despawn the parent cube, just the children
    if event.listener() != event.target() {
        commands.entity(event.target()).despawn();
        info!("I deleted the thing!");
    }
}

/// A forwarded event, an alternative to using callbacks.
struct Greeting(Entity);
impl ForwardedEvent<PointerOver> for Greeting {
    fn from_data(event_data: &EventData<PointerOver>) -> Greeting {
        Greeting(event_data.target())
    }
}
impl Greeting {
    fn handle_events(mut greet: EventReader<Greeting>) {
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
            // When any of this entity's children are clicked, they will be deleted
            EventListener::<PointerClick>::callback(delete_me),
        ))
        // Because event forwarding uses bubbling, events that target children of the parent cube
        // will bubble up to this level and will fire off a `Greeting` event.
        .forward_events::<PointerOver, Greeting>()
        .with_children(|parent| {
            for i in 1..=5 {
                parent.spawn((
                    // As noted above, we are adding children here but we don't need to add an event
                    // listener. Events on children will bubble up to the parent!
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.4 })),
                        material: materials.add(Color::RED.into()),
                        transform: Transform::from_xyz(0.0, 1.0 + 0.5 * i as f32, 0.0),
                        ..Default::default()
                    },
                    PickableBundle::default(),
                    PickRaycastTarget::default(),
                ));
            }
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
            transform: Transform::from_xyz(-2.0, 4.5, 5.0).looking_at(Vec3::Y * 2.0, Vec3::Y),
            ..Default::default()
        },
        PickRaycastSource::default(),
    ));
}
