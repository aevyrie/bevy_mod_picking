//! This example demonstrates how [`OnPointer`] components and event bubbling can be used to
//! propagate events up an entity hierarchy, and run callbacks when an event reaches an entity.
//!
//! The Big Idea here is to make it easy to couple interaction events with specific entities. In
//! other words, it allows you to easily implement "If entity X is hovered/clicked/dragged, do Y".

use bevy::prelude::*;
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DefaultHighlightingPlugin>(),
        )
        .add_plugins(bevy_egui::EguiPlugin)
        .add_systems(Startup, setup)
        .add_event::<DoSomethingComplex>()
        .add_systems(
            Update,
            receive_greetings.run_if(on_event::<DoSomethingComplex>()),
        )
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
            // Callbacks are just exclusive bevy systems that have access to an event data via
            // `](bevy_eventlistener::prelude::Listener) and [`ListenerMut`]. This gives
            // you full freedom to write normal bevy
            // systems that are only called when specific entities are interacted with. Here we have
            // a system that rotates a cube when it is dragged. See the comments added to the
            // function for more details on the requirements of callback systems.
            //
            // # Performance 💀
            //
            // Callback systems require exclusive world access, which means the system cannot be run
            // in parallel with other systems! Callback systems are very flexible, but should be
            // used with care. If you want to do something complex in response to a listened event,
            // prefer to instead use `send_event`, and react to your custom event in a
            // normally-scheduled bevy system (see send_event usage below).
            On::<Pointer<Move>>::run(change_hue_with_vertical_move),
            // We can use helper methods to make callbacks even simpler. For drag-to-rotate, we use
            // this little closure, because we only need to modify the target entity's Transform:
            On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                transform.rotate_local_y(drag.delta.x / 50.0)
            }),
            // Just like bevy systems, callbacks can be closures! Recall that the parameters can be
            // any bevy system parameters, with the only requirement that the first parameter be the
            // input event, and the function output is a `Bubble`.
            On::<Pointer<Out>>::run(|event: Listener<Pointer<Out>>, time: Res<Time>| {
                info!(
                    "[{:?}]: The pointer left entity {:?}",
                    time.elapsed_seconds(),
                    event.target
                );
            }),
            // When you just want to add a `Command` to the target entity,`add_target_commands` will
            // reduce boilerplate and allow you to do this directly.
            On::<Pointer<Click>>::target_commands_mut(|click, target_commands| {
                if click.target != click.listener() && click.button == PointerButton::Secondary {
                    target_commands.despawn();
                }
            }),
            // Sometimes you may need to do multiple things in response to an interaction. Events
            // can be an easy way to handle this, as you can react to an event across many systems.
            // Unlike pointer events, recall that this event is only sent when the event listener
            // for this *specific* entity (or its children) are targeted. Similar to `add_command`
            // this is simply a helper function that creates an event-sending callback to reduce
            // boilerplate.
            //
            // # Performance 🚀
            //
            // Unlike the `run` method, this will not prevent systems from parallelizing, as the
            // systems that react to this event can be scheduled normally. In fact, you can get the
            // best of both worlds by using run criteria on the systems that react to your custom
            // event. This allows you to run bevy systems in response to interaction with a specific
            // entity, while still allowing full system parallelism.
            On::<Pointer<Down>>::send_event::<DoSomethingComplex>(),
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

/// Change the hue of mesh's `StandardMaterial` when the mouse moves vertically over it.
fn change_hue_with_vertical_move(
    // The event data accessible by the callback system
    event: Listener<Pointer<Move>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube: Query<&Handle<StandardMaterial>>,
) {
    let material = materials.get_mut(cube.get(event.target).unwrap()).unwrap();
    let mut color = material.base_color.as_hsla_f32();
    let to_u8 = 255.0 / 360.0; // we will use wrapping integer addition to make the hue wrap around
    color[0] = ((color[0] * to_u8) as u8).wrapping_add_signed(event.delta.y as i8) as f32 / to_u8;
    material.base_color = Color::hsla(color[0], color[1], color[2], color[3]);
}

struct DoSomethingComplex(Entity, f32);

impl From<ListenerInput<Pointer<Down>>> for DoSomethingComplex {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        DoSomethingComplex(event.target, event.hit.depth)
    }
}

/// Unlike callback systems, this is a normal system that can be run in parallel with other systems.
fn receive_greetings(mut greetings: EventReader<DoSomethingComplex>) {
    for event in greetings.iter() {
        info!(
            "Hello {:?}, you are {:?} depth units away from the pointer",
            event.0, event.1
        );
    }
}
