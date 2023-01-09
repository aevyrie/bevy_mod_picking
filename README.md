<div align="center">
  
# Mouse Picking for Bevy

[![crates.io](https://img.shields.io/crates/v/bevy_mod_picking)](https://crates.io/crates/bevy_mod_picking)
[![docs.rs](https://docs.rs/bevy_mod_picking/badge.svg)](https://docs.rs/bevy_mod_picking)
[![CI](https://github.com/aevyrie/bevy_mod_picking/workflows/CI/badge.svg?branch=master)](https://github.com/aevyrie/bevy_mod_picking/actions?query=workflow%3A%22CI%22+branch%3Amaster)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

![picking_demo](https://user-images.githubusercontent.com/2632925/201516335-9ee66106-b2d8-4eb5-bc32-18da072999a2.gif)

A [Bevy](https://github.com/bevyengine/bevy) plugin for picking, making it easy to interact
with meshes in Bevy. Built with [`bevy_mod_raycast`](https://github.com/aevyrie/bevy_mod_raycast).

</div>

## Features
* Mouse intersection coordinates in world space
* Mouseover and mouseclick events
* Configurable highlighting
* Selection state management
* 3D debug cursor
* Touch support
* Common keybindings (Ctrl+A, Ctrl+Click multi-select)

# Quickstart

It only takes a few lines to get mouse picking working in your Bevy application using this plugin.

1. Add the plugin to your app:
```rs
.add_plugins(DefaultPickingPlugins);
```

2. Mark your camera as the picking source with the `PickingCameraBundle` component:
```rs
.insert(PickingCameraBundle::default());
```

3. Add the `PickableBundle` component to any meshes you want to make pickable:
```rs
.insert(PickableBundle::default())
```

That's all there is to it! Read [the docs](https://docs.rs/bevy_mod_picking) and look at the provided examples to learn more.

# Demo

To run a minimal demo, clone this repository and run:

```console
cargo run --example minimal 
```

# Bevy Version Support

I intend to track the `main` branch of Bevy. PRs supporting this are welcome!

| bevy | bevy_mod_picking |
| ---- | ---------------- |
| 0.9  | 0.10, 0.11       |
| 0.8  | 0.8, 0.9         |
| 0.7  | 0.6, 0.7         |
| 0.6  | 0.5              |
| 0.5  | 0.4              |
| 0.4  | 0.3              |
| 0.3  | 0.2              |

# License

This project is licensed under the [MIT license](https://github.com/aevyrie/bevy_mod_picking/blob/master/LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bevy_mod_picking by you, shall be licensed as MIT, without any additional terms or conditions.
