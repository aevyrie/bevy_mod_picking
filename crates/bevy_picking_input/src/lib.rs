//! `bevy_picking_input` is a thin layer that provides unsurprising default inputs to `bevy_picking
//! core`. The included systems are responsible for sending  mouse and touch inputs to their
//! respective `Pointer`s.
//!
//! Because this resides in its own crate, it's easy to omit it, and provide your own inputs as
//! needed. Because `Pointer`s aren't coupled to the underlying input hardware, you can easily mock
//! inputs, and allow users full accessibility to map whatever inputs they need to pointer input.
//!
//! If, for example, you wanted to add support for VR input, all you need to do is spawn a pointer
//! entity with a custom [`PointerId`](bevy_picking_core::pointer::PointerId), and write a system
//! that updates its position.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_picking_core::{PickSet, PickingPluginsSettings};

pub mod debug;
pub mod mouse;
pub mod touch;

/// Adds mouse and touch inputs for picking pointers to your app.
pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputPluginSettings>()
            .add_systems(Startup, mouse::spawn_mouse_pointer)
            .add_systems(
                First,
                (
                    touch::touch_pick_events.run_if(touch_enabled),
                    mouse::mouse_pick_events.run_if(mouse_enabled),
                    // IMPORTANT: the commands must be flushed after `touch_pick_events` is run
                    // because we need pointer spawning to happen immediately to prevent issues with
                    // missed events during drag and drop.
                    apply_deferred,
                )
                    .chain()
                    .in_set(PickSet::Input),
            )
            .add_systems(
                Last,
                touch::deactivate_pointers.run_if(PickingPluginsSettings::input_enabled),
            );
    }
}

/// Settings for the input plugin to allow enabling or disabling mouse or touch inputs at runtime.
#[derive(Resource)]
pub struct InputPluginSettings {
    run_mouse: bool,
    run_touch: bool,
}
impl Default for InputPluginSettings {
    fn default() -> Self {
        Self {
            run_mouse: true,
            run_touch: true,
        }
    }
}

fn touch_enabled(settings: Res<InputPluginSettings>, state: Res<PickingPluginsSettings>) -> bool {
    state.enable && state.enable_input && settings.run_touch
}
fn mouse_enabled(settings: Res<InputPluginSettings>, state: Res<PickingPluginsSettings>) -> bool {
    state.enable && state.enable_input && settings.run_mouse
}
