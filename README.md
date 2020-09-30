# 3D Mouse Picking for Bevy

![](https://img.shields.io/github/workflow/status/aevyrie/bevy_mod_picking/Continuous%20integration)

This is a 3D mouse picking plugin for [Bevy](https://github.com/bevyengine/bevy). The plugin will cast a ray into the scene and check for intersection against all meshes tagged with the `PickableMesh` component. The built-in highlighting and selection state, as well as the debug cursor, are opt-in.

Out of the box, the plugin provides: pick depth, pick coordinates, and surface normal of the picked mesh triangle.

**Expect Breaking Changes in `master` - Contributions Welcome**

![Picking demo](https://raw.githubusercontent.com/aevyrie/bevy_mod_picking/master/docs/picking_demo.webp)

## Getting Started

To run the `3d_scene` example - a modified version of the `Bevy` example of the same name - clone this repository and run:

```console
cargo run --example 3d_scene
```

## Usage

### Setup

Add the plugin to your dependencies in Cargo.toml

```toml
bevy_mod_picking = { git = "https://https://github.com/aevyrie/bevy_mod_picking", branch = "master" }
#bevy_mod_picking = "0.1.2"
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

For simple use cases, you will probably be using the mouse to pick items in a 3d scene. You will need to mark your camera with a component:

```rust
.with(PickingSource::default())
```

Now all you have to do is mark any mesh entities with the `PickableMesh` component:

```rust
.with(PickableMesh::default())
```

If you want it to highlight when you hover, add the `HighlightablePickMesh` component:

```rust
.with(HighlightablePickMesh::new())
```

If you also want to select meshes and keep them highlighted with the left mouse button, add the `SelectablePickMesh` component:

```rust
.with(SelectablePickMesh::new())
```

### Pick Groups

Pick groups allow you to associate meshes with a ray casting sources. For simple use cases, such as a single 3d view and camera, you can ignore this.

For these simple cases, you can just use `PickingGroup::default()` any time a `PickingGroup` is required. This will assign the `PickableMesh` or `PickingSource` to picking group 0.

#### Details

 - Only one PickingSource can be assigned to a PickingGroup
 - A PickableMesh can be assigned to one or many PickingGroups
 - The result of running the picking system is an ordered list of all intersections of each PickingSource with the PickableMeshs in its PickingGroup. The ordered list of intersections are stored by PickingGroup `HashMap<PickingGroup, Vec<PickIntersection>>`

### Getting Pick Data

#### Pick Intersections Under the Cursor

Mesh picking intersection are reported in world coordinates. You can use the `PickState` resource to either get the topmost entity, or a list of all entities sorted by distance (near -> far) under the cursor:

```rust
fn get_picks(
    pick_state: Res<PickState>,
) {
    println!("All entities:\n{:?}", pick_state.list(PickingGroup::default()));
    println!("Top entity:\n{:?}", pick_state.top(PickingGroup::default()));
}
```
#### Pick Interactions

A InteractableMesh Plugin has been provided that will provide events such as mouse_entered, mouse_exited, mouse_down(MouseButton), mouse_just_pressed / mouse_just_released. You can view the implementations in interactable_cube.rs

#### Selection State

If you're using the `SelectablePickMesh` component for selection, you can access the selection state by querying all selectable entities and accessing the `.selected()` function.

### Plugin Parameters

If you're using the built in `HighlightablePickMash` component for highlighting, you can change the colors by accessing the `PickHighlightParams` and setting the colors:

```rust
fn set_highlight_params(
    mut highlight_params: ResMut<PickHighlightParams>,
) {
    highlight_params.set_hover_color(Color::rgb(1.0, 0.0, 0.0));
    highlight_params.set_selection_color(Color::rgb(1.0, 0.0, 1.0));
}
```

### Debug

You can also enable a debug cursor that will place a sphere at the intersection, with a tail pointing normal to the surface. Just add the `DebugPickingPlugin` to the `App::build()` in your Bevy program:

```rust
.add_plugin(DebugPickingPlugin)
```
