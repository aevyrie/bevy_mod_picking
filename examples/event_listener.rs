//! This example demonstrates how [`EventListener`]s and event bubbling can be used to propagate
//! events up an entity hierarchy, and run callbacks when an event reaches an entity.
//!
//! The Big Idea here is to make it easy to couple interaction events with specific entities. In
//! other words, it allows you to easily implement "If entity X is hovered/clicked/dragged, do Y".
//!
//! The `forward_events` function might seem like magic, but it's pretty straightforward under the
//! hood. It simply adds an [`EventListener`] component to the entity. When the event bubbling
//! system encounters this `EventListener`, it uses the [`ForwardedEvent`] trait you implemented on
//! your custom event to convert the `PointerEvent` into your custom event.
//!
//! In other words, the `forward_events` function is really a helper function that just inserts a
//! predefined `EventListener` component on your entity.

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
        .add_event::<Greeting>()
        .add_system(Greeting::print_events)
        .run();
}

/// A callback function used with an `EventListener`.
fn delete_target(commands: &mut Commands, event: &EventListenerData<Click>, _: &mut Bubble) {
    // We don't want to despawn the parent cube, just the children
    if event.listener != event.target {
        commands.entity(event.target).despawn();
        info!("I deleted {:?}!", event.target);
    }
}

/// A forwarded event, an alternative to using callbacks.
struct Greeting(Entity);
impl ForwardedEvent<Over> for Greeting {
    fn from_data(event_data: &EventListenerData<Over>) -> Greeting {
        Greeting(event_data.target)
    }
}
impl Greeting {
    fn print_events(mut greet: EventReader<Greeting>) {
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
            RaycastPickTarget::default(),
            // When any of this entity's children are clicked, they will be deleted
            EventListener::<Click>::callback(delete_target),
            // `forward_event` is a special case of `callback` that simply sends a user event when a
            // specific pointer event reaches this entity. In this case, when a pointer over event
            // occurs for any children of this entity, a `Greeting` event will be sent.
            EventListener::<Over>::forward_event::<Greeting>(),
        ))
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
                    RaycastPickTarget::default(),
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
        RaycastPickCamera::default(),
    ));
}
