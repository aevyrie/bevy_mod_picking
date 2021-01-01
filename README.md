# 3D Mouse Picking for Bevy

[![CI](https://img.shields.io/github/workflow/status/aevyrie/bevy_mod_picking/Continuous%20integration/master)](https://github.com/aevyrie/bevy_mod_picking/actions?query=workflow%3A%22Continuous+integration%22+branch%3Amaster)
[![dependency status](https://deps.rs/repo/github/aevyrie/bevy_mod_picking/status.svg)](https://deps.rs/repo/github/aevyrie/bevy_mod_picking)
[![crates.io](https://img.shields.io/crates/v/bevy_mod_picking)](https://crates.io/crates/bevy_mod_picking)
[![docs.rs](https://docs.rs/bevy_mod_picking/badge.svg)](https://docs.rs/bevy_mod_picking)

A [Bevy](https://github.com/bevyengine/bevy) plugin for 3D mouse picking, making it easy to interact with 3D geometry using your mouse or any other ray casting source! The plugin includes built-in mouse events, highlighting, selection state, multi-window support, and a debug cursor, all of which are opt-in.

**Expect breaking changes in `master` branch - contributions are welcome!**

![Picking demo](https://raw.githubusercontent.com/aevyrie/bevy_mod_picking/master/docs/picking_demo.webp)

## Features
* Multiple picking sources
    * Mouse (relative to supplied camera)
    * Screen space coordinates (relative to supplied camera)
    * Manually defined ray using a transform
* [Pick Data](#getting-pick-data)
    * Pick depth
    * Pick coordinates (in world space)
    * Surface normal at the pick ray intersection
    * Vertex coordinates of the intersected triangle
* [Mesh Interaction](#interacting-with-meshes)
    * Mouseover and click events
    * Configurable color highlighting for hovering and selection
    * Mesh selection state management
* [3D Debug cursor to show pick intersection and surface normal](#debug)
* [Picking Groups](#pick-groups) (associate a picking source with a set of meshes)
    * Multi window support

## Demo

To run the `3d_scene` example - a modified version of the `Bevy` example of the same name - clone this repository and run:

```console
cargo run --example 3d_scene --features="example_deps"
```

Note that by default this plugin only depends on bevy's `render` feature to minimize dependency count and build time, and allow for wasm support. This is why the feature flag is needed to run examples, which need the winit and wgpu features to run.

# Getting Started

It only takes a few lines to get mouse picking working in your Bevy application using this plugin; [check out the 3d_scene example](https://github.com/aevyrie/bevy_mod_picking/blob/master/examples/3d_scene.rs) for a minimal implementation and demo! The following sections will walk you through what is needed to get the plugin working, and how everything fits together.

## Setup

Add the plugin to your dependencies in Cargo.toml

```toml
bevy_mod_picking = "0.3.1"
```

Import the plugin:

```rust
use bevy_mod_picking::*;
```

Add it to your App::build() in the plugins section of your Bevy app:

```rust
.add_plugin(PickingPlugin)
```

### Marking Entities for Picking

For simple use cases, you will probably be using the mouse to pick items in a 3d scene. You will can mark your camera with a default PickSource component:

```rust
.with(PickSource::default())
```

Now all you have to do is mark any mesh entities with the `PickableMesh` component:

```rust
.with(PickableMesh::default())
```

## Interacting with Meshes

To get mouseover and mouseclick events, as well as built-in highlighting and selection state, you will need to add the `InteractablePickingPlugin` plugin. This is intentionally left optional, in case you only need pick intersection results.

```rust
// Add this below the PickingPlugin line
.add_plugin(InteractablePickingPlugin)
```

See the [Pick Interactions](#pick-interactions) section for more details on the features this provides.
You will need to add the `InteractableMesh` component to entities to use these features.

```rust
.with(PickableMesh::default())
.with(InteractableMesh::default())
```

If you want a mesh to highlight when you hover, add the `HighlightablePickMesh` component:

```rust
// InteractableMesh component is a prerequisite for this to work
.with(HighlightablePickMesh::default())
```

If you also want to select meshes and keep them highlighted when clicked with the left mouse button, add the `SelectablePickMesh` component:

```rust
// InteractableMesh component is a prerequisite for this to work
.with(SelectablePickMesh::default())
```

## Pick Groups

Pick groups allow you to associate meshes with a ray casting source, and produce a pick result for each group. For simple use cases, such as a single 3d view and camera, you can ignore this.

For those simple cases, you can just use `Group::default()` any time a `Group` is required. This will assign the `PickableMesh` or `PickSource` to picking group 0.

Pick groups are useful in cases such as multiple windows, where you want each window to have its own picking source (cursor relative to that window's camera), and each window might have a different set of meshes that this picking source can intersect. The primary window might assign the camera and all relavent meshes to pick group 0, while the secondary window uses pick group 1 for these. See the [multiple_windows](https://github.com/aevyrie/bevy_mod_picking/blob/master/examples/multiple_windows.rs) example for implementation details.

### Constraints

- Only one PickSource can be assigned to a `Group`
- A PickableMesh can be assigned to one or more `Group`s
- The result of running the picking system is an ordered list of all intersections of each `PickSource` with the `PickableMesh`s in that `Group`. The ordered list of intersections are stored by `Group`, `HashMap<Group, Vec<PickIntersection>>`

# Getting Pick Data

Mesh picking intersection are reported in world coordinates. You can use the `PickState` resource to either get the topmost entity, or a list of all entities sorted by distance (near -> far) under the cursor:

```rust
fn get_picks(
    pick_state: Res<PickState>,
) {
    println!("All entities:\n{:?}", pick_state.list(Group::default()));
    println!("Top entity:\n{:?}", pick_state.top(Group::default()));
}
```

Alternatively, and perhaps more idiomatic to the Bevy ECS system, you can get the intersections for entities that have the `PickableMesh` component using:

```rust
pickable_entity.intersection(Group::default());
```

## Pick Interactions

Run the `events` example to see mouseover and mouseclick events in action:

```console
cargo run --example events
```

The `InteractableMesh` component stores mouseover event state, mouseclick event state (left, right, and middle buttons), and hover state.

## Selection State

If you're using the `SelectablePickMesh` component for selection, you can access the selection state by querying your selectable entities and accessing the `.selected()` function.

## Plugin Parameters

If you're using the built in `HighlightablePickMash` component for highlighting, you can change the colors by accessing the `PickHighlightParams` and setting the colors:

```rust
// Example Bevy system to set the highlight colors
fn set_highlight_params(
    mut highlight_params: ResMut<PickHighlightParams>,
) {
    highlight_params.set_hover_color(Color::rgb(1.0, 0.0, 0.0));
    highlight_params.set_selection_color(Color::rgb(1.0, 0.0, 1.0));
}
```

# Debug

You can enable a debug cursor that will place a sphere at the intersection, with a tail pointing normal to the surface. Just add the `DebugPickingPlugin` to the `App::build()` in your Bevy program:

```rust
.add_plugin(DebugPickingPlugin)
```

# Bounding Sphere Optimization

This plugin has the ability to accelerate picking with bounding spheres; this can make picking as much as **30 times faster**! This speeds up the picking process by first checking to see if the picking source intersects a mesh's bounding sphere before going through every triangle in the mesh. To enable bounding spheres, you can use the builder pattern to pass a handle to your mesh into the `.with_bounding_sphere()` function:

```rust
.with(PickableMesh::default()
    .with_bounding_sphere(mesh_handle)
);
```

You can see an example of this in action in the `stress_test` example in this repo. Please be aware that the API for this feature is likely to change over coming releases.

# License

This project is licensed under the [MIT license](https://github.com/aevyrie/bevy_mod_picking/blob/master/LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bevy_mod_picking by you, shall be licensed as MIT, without any additional terms or conditions.
