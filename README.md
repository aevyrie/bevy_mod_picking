# 3D Mouse Picking for Bevy

[![CI](https://github.com/aevyrie/bevy_mod_picking/workflows/CI/badge.svg?branch=master)](https://github.com/aevyrie/bevy_mod_picking/actions?query=workflow%3A%22CI%22+branch%3Amaster)
[![crates.io](https://img.shields.io/crates/v/bevy_mod_picking)](https://crates.io/crates/bevy_mod_picking)
[![docs.rs](https://docs.rs/bevy_mod_picking/badge.svg)](https://docs.rs/bevy_mod_picking)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

A [Bevy](https://github.com/bevyengine/bevy) plugin for 3D mouse picking, making it easy to
interact with 3D geometry in Bevy using your mouse. The plugin provides mouse intersection coordinates, a number of built-in mouse
events, highlighting, selection state, and a 3D debug cursor. This plugin is build on top of [`bevy_mod_raycast`](https://github.com/aevyrie/bevy_mod_raycast).

**Expect breaking changes in `master` branch - contributions are welcome!**


![Picking demo](https://user-images.githubusercontent.com/2632925/111893615-3a9d1a80-89c1-11eb-980f-9c546df990f5.png)

## Features
* [Pick Data](#getting-pick-data): intersection surface normal and coordinates in world space
* [Mesh Interaction](#interacting-with-meshes): mouseover and mouseclick, highlighting, selection state
* [Debug cursor](#debug): debug pick intersections and surface normals with a 3d cursor

## Bevy Version Support

I intend to track the `main` branch of Bevy. PRs supporting this are welcome! 

|bevy|bevy_mod_picking|
|---|---|
|0.5|0.4|
|0.4|0.3|
|0.3|0.2|

## Demo

To run the `3d_scene` example - a modified version of the `Bevy` example of the same name - clone this repository and run:

```console
cargo run --example 3d_scene --features="example_deps"
```

Note that by default this plugin only depends on bevy's `render` feature to minimize both dependency count and compile time, as well as allow for wasm support. This is why the feature flag is needed to run examples, which need the winit and wgpu features to run.

# Quickstart

It only takes a few lines to get mouse picking working in your Bevy application using this plugin. The following sections will walk you through what is needed to get the plugin working, and how everything fits together.

1. Add the plugin to your dependencies in Cargo.toml
```toml
bevy_mod_picking = "0.4"
```

2. Import the plugin and add it to your Bevy app:
```rust
use bevy_mod_picking::*;
// Bevy app stuff here...
.add_plugin(PickingPlugin)
```

3. Mark your camera with a `PickingCameraBundle`; this tells the plugin what camera you are using to render to the screen:
```rust
.with_bundle(PickingCameraBundle::default())
```

4. Now all you have to do is add the `PickableBundle` to your meshes to make them "pickable":
```rust
.with_bundle(PickableBundle::default())
```

And that's it! To learn how to retreive picking intersections, you can jump to the [Getting Pick Data](#getting-pick-data) section. If you also need interaction features, e.g. mouseclick & mousehover events, highlighting, and selection state, continue reading.

# Interacting with Meshes

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

# Getting Pick Data

Mesh picking intersections are reported in world coordinates. A ray is cast into the scene using
the `PickSource` you provided, and checked for intersections against every mesh that has been
marked as a `PickableMesh`. The results report which entities were intersected, as well as the 3D
coordinates of the corresponding intersection.

To access this data, you can query your picking camera, and use `.intersect_list()` or `.intersect_top()`.

## Pick Interactions
 
Run the `events` example to see mouseover and mouseclick events in action:

```shell
cargo run --example events
```

## Selection State

If you're using the `Selection` component for selection, you can access the selection state by querying your selectable entities and accessing the `.selected()` function.

## Plugin Parameters

If you're using the built in `HighlightablePickMash` component for highlighting, you can change the colors by accessing the `PickHighlightParams` and setting the colors:

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

This will run a system in Bevy to automatically compute the bounding sphere of the supplied mesh.You can see an example of bounding spheres used in the `stress_test` example. Please be aware that the API for this feature is likely to change over coming releases.

# License

This project is licensed under the [MIT license](https://github.com/aevyrie/bevy_mod_picking/blob/master/LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bevy_mod_picking by you, shall be licensed as MIT, without any additional terms or conditions.
