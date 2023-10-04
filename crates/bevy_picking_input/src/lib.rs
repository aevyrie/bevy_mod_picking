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

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

use bevy_picking_core::PickSet;

pub mod debug;
pub mod mouse;
pub mod touch;

/// Common imports for `bevy_picking_input`.
pub mod prelude {
    pub use crate::{InputPlugin, InputPluginSettings};
}

/// Adds mouse and touch inputs for picking pointers to your app.
pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputPluginSettings>()
            .add_systems(Startup, mouse::spawn_mouse_pointer)
            .add_systems(
                First,
                (
                    touch::touch_pick_events.run_if(InputPluginSettings::is_touch_enabled),
                    mouse::mouse_pick_events.run_if(InputPluginSettings::is_mouse_enabled),
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
                touch::deactivate_touch_pointers.run_if(InputPluginSettings::is_touch_enabled),
            );
    }
}

/// A resource used to enable and disable features of the [`InputPlugin`].
#[derive(Resource, Debug, Reflect)]
pub enum InputPluginSettings {
    /// The plugin is enabled and systems will run, even if all the inner fields corresponding to
    /// specific features are disabled.
    Enabled {
        /// Should touch inputs be updated?
        is_touch_enabled: bool,
        /// Should mouse inputs be updated?
        is_mouse_enabled: bool,
    },
    /// Completely disable the plugin.
    Disabled,
}

impl Default for InputPluginSettings {
    fn default() -> Self {
        Self::Enabled {
            is_touch_enabled: true,
            is_mouse_enabled: true,
        }
    }
}

impl InputPluginSettings {
    fn is_touch_enabled(state: Res<Self>) -> bool {
        matches!(
            *state,
            Self::Enabled {
                is_touch_enabled: true,
                ..
            }
        )
    }
    fn is_mouse_enabled(state: Res<Self>) -> bool {
        matches!(
            *state,
            Self::Enabled {
                is_mouse_enabled: true,
                ..
            }
        )
    }

    /// Returns `true` if the input plugin state is [`Enabled`].
    ///
    /// [`Enabled`]: InputPluginSettings::Enabled
    pub fn is_enabled(state: Res<Self>) -> bool {
        matches!(*state, Self::Enabled { .. })
    }
}
