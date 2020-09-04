# bevy_mod_picking

3D mouse picking prototype for Bevy. Casts a ray into the scene and checks for intersection for all meshes tagged with the `PickableMesh` component.

**Super duper WIP - Issues Welcome**

![Picking demo](docs/demo.gif)

## Getting Started

Run the 3d_scene example, a modified version of the `bevy` example of the same name, with:
```console
cargo run --example 3d_scene
```

## Usage

Add the repo to your dependencies in Cargo.toml

```toml
bevy_mod_picking = { git = "https://github.com/aevyrie/bevy_mod_picking", branch = "master" }
```

Import the plugin:

```rust
use bevy_mod_picking::*;
```

Add it to your App::build() in the plugins section:

```rust
.add_plugin(PickingPlugin)
```

Make sure you have your camera's entity on hand, you could do the following in your setup system:

```rust
let camera_entity = Entity::new();

// ...

.spawn_as_entity(camera_entity, Camera3dComponents {

// ...
```

Now all you have to do is mark any mesh entities with the `PickableMesh` component:

```rust
.with(PickableMesh::new(camera_entity))
```

If you want it to highlight when you hover, add the `HighlightablePickMesh` component:

```rust
.with(HighlightablePickMesh::new())
```

If you also want to select meshes and keep them highlighted with the left mouse button, add the `SelectablePickMesh` component:

```rust
.with(SelectablePickMesh::new())
```

If you want to get the entities that are being hovered over, you can use the `PickState` resource to either get the topmost entity, or a sorted list of all entities under the cursor:

```rust
fn get_picks(
    pick_state: Res<PickState>,
) {
    println!("All entities:\n{:?}", pick_state.list());
    println!("Top entity:\n{:?}", pick_state.top());
}
```

You can also iterate over all `PickableMesh`s, and read the `picked` feild. (This isn't yet publically exposed, and will probably change to a DepthPick struct).

### Limitations

Current limitations I'd like to fix:

* Single camera and window: eventually I'd like to support picking for an arbitrary number of cameras and windows. The camera entity currently passed into new `PickableMesh` instances does not yet do anything.

* No orthographic camera support: this hasn't been tested yet, but is an explicit goal of this plugin.

* Optimizations: the current ray casting implementation is naive, and queries all meshes in the scene. The first optimization I'd like to apply is checking against bounding spheres before checking each triangle in a mesh. This should greatly improve performance in cases where the cursor is hovering over an area with very few objects.

* No "color-picking" implementation: for performance, I'd like to render the scene to an off-screen buffer that renders each pixel as a mesh ID encoded into RGBA. Picking is then as simple as querying this buffer and doing a lookup to return a mesh handle.

* Fixed ~~Highlighting coupled with ray casting system~~
