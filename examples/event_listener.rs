//! This example demonstrates how [`OnPointer`] components and event bubbling can be used to
//! propagate events up an entity hierarchy, and run callbacks when an event reaches an entity.
//!
//! The Big Idea here is to make it easy to couple interaction events with specific entities. In
//! other words, it allows you to easily implement "If entity X is hovered/clicked/dragged, do Y".

use bevy::{ecs::system::Command, input::mouse::MouseMotion, prelude::*};
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
        .add_system(receive_greetings)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        // When any of this entity's children are interacted with using a pointer, those events will
        // propagate up the entity hierarchy until they reach this parent. By referring to the
        // `target` entity instead of the `listener` entity, we can do things to specific target
        // entities, even though they lack `OnPointer` components.
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::WHITE.into()),
                ..Default::default()
            },
            PickableBundle::default(),
            RaycastPickTarget::default(),
            // Callbacks are just bevy systems that have a specific input (`In<ListenedEvent<E>>`)
            // and output (`Bubble`). This gives you full freedom to write normal bevy systems that
            // are only called when specific entities are interacted with. Here we have a system
            // that rotates a cube when it is dragged, in just a few lines of code:
            OnPointer::<Drag>::run_callback(rotate_with_mouse),
            // Just like bevy systems, callbacks can be closures!
            OnPointer::<Out>::run_callback(|In(event): In<ListenedEvent<Out>>| {
                info!("The pointer left entity {:?}", event.target);
                Bubble::Up
            }),
            // When you just want to do something simple, the `add_command` helper will handle the
            // boilerplate of adding a bevy `Command`. Because it's added with a closure, this
            // allows us to pass event data into the command:
            OnPointer::<Click>::add_command::<DeleteTarget>(),
            // Sometimes you may need to do multiple things in response to an interaction. Events
            // can be an easy way to handle this, as you can react to an event across many systems.
            // Unlike pointer events, recall that this event is only sent when the event listener
            // for this *specific* entity (or its children) are targeted. Similar to `add_command`
            // this is simply a helper function that creates an event-sending callback to reduce
            // boilerplate.
            OnPointer::<Over>::send_event::<Greeting>(),
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

/// Delete the entity if clicked with RMB and not the root entity
struct DeleteTarget(Entity, PointerButton);

impl From<ListenedEvent<Click>> for DeleteTarget {
    fn from(event: ListenedEvent<Click>) -> Self {
        DeleteTarget(event.target, event.pointer_event.button)
    }
}

impl Command for DeleteTarget {
    fn write(self, world: &mut World) {
        let target = world.entity_mut(self.0);
        if target.get::<Children>().is_none() && self.1 == PointerButton::Secondary {
            target.despawn();
        }
    }
}

/// Rotate the target entity about its y axis.
fn rotate_with_mouse(
    // The first parameter is always the `ListenedEvent`, passed in by the event listening system.
    In(event): In<ListenedEvent<Drag>>,
    // The following can be any normal bevy system params:
    mut mouse_move: EventReader<MouseMotion>,
    mut cube: Query<&mut Transform>,
) -> Bubble {
    let total_drag_dist: f32 = mouse_move.iter().map(|mm| mm.delta.x).sum();
    if let Ok(mut transform) = cube.get_mut(event.target) {
        transform.rotate_local_y(total_drag_dist / 50.0);
    }
    Bubble::Up // Determines if the event should continue to bubble through the hierarchy.
}

struct Greeting(Entity, f32);

impl From<ListenedEvent<Over>> for Greeting {
    fn from(event: ListenedEvent<Over>) -> Self {
        Greeting(event.target, event.pointer_event.hit.depth)
    }
}

fn receive_greetings(mut greetings: EventReader<Greeting>) {
    for event in greetings.iter() {
        info!(
            "Hello {:?}, you are {:?} depth units away from the pointer",
            event.0, event.1
        );
    }
}
