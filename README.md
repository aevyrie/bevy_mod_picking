# bevy_mod_picking

3D mouse picking plugin prototype for Bevy. Casts a ray into the scene and checks for intersection against all meshes tagged with the `PickableMesh` component. Included highlighting and selection state management features are opt-in.

**Super duper WIP - Issues Welcome**

![Picking demo](docs/demo.gif)

## Getting Started

To run the 3d_scene example, a modified version of the `bevy` example of the same name, clone this repository and run:
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
