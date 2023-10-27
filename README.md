<div align="center">

# Picking and Pointer Events for Bevy

[![crates.io](https://img.shields.io/crates/v/bevy_mod_picking)](https://crates.io/crates/bevy_mod_picking)
[![docs.rs](https://docs.rs/bevy_mod_picking/badge.svg)](https://docs.rs/bevy_mod_picking)
[![CI](https://github.com/aevyrie/bevy_mod_picking/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/aevyrie/bevy_mod_picking/actions?query=workflow%3A%22CI%22+branch%3Amain)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

![demo](https://user-images.githubusercontent.com/2632925/235874600-de0c7720-6775-42e1-8650-41ee8ac68d1b.gif)

A flexible set of plugins that add picking functionality to your [`bevy`](https://github.com/bevyengine/bevy) app. Want to drag a UI
entity and drop it onto a 3D mesh entity? This plugin allows you to add event listeners to **any**
entity, and works with mouse, touch, or even gamepads.

</div>

# Highlights

- ***Lightweight***: only compile what you need.
- ***Expressive***: event listener components `On::<Pointer<Click>>::run(my_system)`.
- ***Input Agnostic***: control pointers with mouse, pen, touch, or custom bevy systems.
- ***Modular Backends***: mix and match backends like `rapier`, `egui`, `bevy_ui`, or write your own.

## Lightweight

Only compile what you use. All non-critical plugins can be disabled, including highlighting,
selection, and any backends not in use. The crate uses no external dependencies unless you need it
for a backend, e.g. `egui` or `rapier`.

## Expressive

The `On::<Pointer<E>>` event listener component makes it easy to react to pointer interactions like
`Click`, `Over`, and `Drag`. Events bubble up the entity hierarchy starting from their target
looking for event listeners, and running any listener's callbacks. These callbacks are normal bevy
systems, though a number of helpers are provided to reduce boilerplate:

```rs
commands.spawn((
    PbrBundle { /* ... */ },
    // These callbacks are run when this entity or its children are interacted with.
    On::<Pointer<Move>>::run(change_hue_with_vertical_move),
    // Rotate an entity when dragged:
    On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
        transform.rotate_local_y(drag.delta.x / 50.0)
    }),
    // Despawn an entity when clicked:
    On::<Pointer<Click>>::target_commands_mut(|_click, target_commands| {
        target_commands.despawn();
    }),
    // Send an event when the pointer is pressed over this entity:
    On::<Pointer<Down>>::send_event::<DoSomethingComplex>(),
));
```

If you don't need event bubbling or callbacks, you can respond to pointer events like you would any
other bevy event, using `EventReader<Pointer<Click>>`, `EventReader<Pointer<Move>>`, etc.

## Input Agnostic

Pointers can be controlled with anything, whether it's the included mouse or touch inputs, or a
custom gamepad input system you write yourself.

## Modular Backends

Picking backends run hit tests to determine if a pointer is over any entities. This plugin provides
an [extremely simple API to write your own backend](crates/bevy_picking_core/src/backend.rs) in
about 100 lines of code; it also includes half a dozen backends out of the box. These include
`rapier`, `egui`, and `bevy_ui`, among others. Multiple backends can be used at the same time! 

You can have a simple rect hit test backend for your UI, a GPU picking shader for your 3D scene, and
this plugin will handle sorting hits and generating events.

## Robust

In addition to these features, this plugin also correctly handles multitouch, multiple windows,
render layers, viewports, and camera order.

# Getting Started

Making objects pickable is pretty straightforward. In the most minimal cases, it's as simple as adding the plugin to your app:

```rs
.add_plugins(DefaultPickingPlugins)
```

and adding the `PickableBundle` to entities that can be picked with the backends you are using:

```rs
commands.spawn((
    PbrBundle::default(),           // The `bevy_picking_raycast` backend works with meshes
    PickableBundle::default(),      // Makes the entity pickable
));
```

You can find a list of built-in backends [here](https://docs.rs/bevy_mod_picking/latest/bevy_mod_picking/backends/index.html)

## Next Steps

To learn more, [read the docs](https://docs.rs/bevy_mod_picking/latest/bevy_mod_picking/) and take a look at the examples in the `/examples` directory. Understanding [bevy_eventlistener](https://github.com/aevyrie/bevy_eventlistener) will also help. Once you are comfortable with that, this crate's `event_listener` example is a great place to start.

# Bevy Version Support

I intend to track the `main` branch of Bevy. PRs supporting this are welcome!

| bevy | bevy_mod_picking |
| ---- | ---------------- |
| 0.11 | 0.15, 0.16       |
| 0.10 | 0.12, 0.13, 0.14 |
| 0.9  | 0.10, 0.11       |
| 0.8  | 0.8, 0.9         |
| 0.7  | 0.6, 0.7         |
| 0.6  | 0.5              |
| 0.5  | 0.4              |
| 0.4  | 0.3              |
| 0.3  | 0.2              |

# License

All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer.

## Your contributions
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

