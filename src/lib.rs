//! A flexible set of plugins for adding picking support to your [`bevy`] app.
//!
//! # Choosing a Picking Backend
//!
//! You will need to choose a picking backend to use. This plugin uses `bevy_mod_raycast` by
//! default; it works with bevy `Mesh`es out of the box and requires no extra dependencies. These
//! qualities make it useful when you are getting started, however it is not particularly performant
//! for large meshes. You should consider switching to the rapier or shader backends if performance
//! becomes a problem. For simple or low-poly games, it may never be an issue.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{app::PluginGroupBuilder, prelude::*, ui::FocusPolicy};

pub use bevy_picking_core::{self as core, focus, output, pointer};
pub use bevy_picking_input as input;

// Optional
#[cfg(feature = "highlight")]
pub use bevy_picking_highlight as highlight;
#[cfg(feature = "selection")]
pub use bevy_picking_selection as selection;

// Backends
#[cfg(feature = "rapier")]
pub use bevy_picking_rapier as rapier;
#[cfg(feature = "mod_raycast")]
pub use bevy_picking_raycast as mod_raycast;
#[cfg(feature = "pick_shader")]
pub use bevy_picking_shader as shader;

/// Common imports
pub mod prelude {
    pub use crate::{
        core::DebugEventsPlugin,
        output::{
            EventData, EventFrom, EventListenerCommands, IsPointerEvent, PointerCancel,
            PointerClick, PointerDown, PointerDrag, PointerDragEnd, PointerDragEnter,
            PointerDragLeave, PointerDragOver, PointerDragStart, PointerDrop, PointerEnter,
            PointerLeave, PointerMove, PointerOut, PointerOver, PointerUp,
        },
        DefaultPickingPlugins, PickableBundle,
    };

    #[cfg(feature = "highlight")]
    pub use crate::highlight::{
        CustomHighlightingPlugin, DefaultHighlighting, HighlightOverride, Highlightable,
        HighlightingPlugins, PickHighlight,
    };

    #[cfg(feature = "selection")]
    pub use crate::selection::{
        NoDeselect, PickSelection, PointerDeselect, PointerMultiselect, PointerSelect,
        SelectionPlugin,
    };

    #[cfg(feature = "mod_raycast")]
    pub use crate::mod_raycast::{PickRaycastSource, PickRaycastTarget};
}

/// A "batteries-included" set of plugins that adds everything needed for picking, highlighting, and
/// multiselect.
pub struct DefaultPickingPlugins;
impl PluginGroup for DefaultPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(core::CorePlugin)
            .add(core::InteractionPlugin)
            .add(input::InputPlugin)
            .add(crate::DefaultPointersPlugin);

        // Optional
        #[cfg(feature = "selection")]
        group.add(selection::SelectionPlugin);
        #[cfg(feature = "highlight")]
        highlight::HighlightingPlugins.build(group);

        // Backends
        #[cfg(feature = "mod_raycast")]
        group.add(mod_raycast::RaycastPlugin);
        #[cfg(feature = "rapier")]
        group.add(rapier::RapierPlugin);
        #[cfg(feature = "pick_shader")]
        group.add(shader::ShaderPickingPlugin);
    }
}

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    /// The entity's configurable [`PickLayer`](focus::PickLayer)
    pub pick_layer: focus::PickLayer,
    /// Tracks entity [`Interaction`] state.
    pub interaction: Interaction,
    /// The entity's configurable [`FocusPolicy`]
    pub focus_policy: FocusPolicy,
    #[cfg(feature = "selection")]
    /// Tracks entity [`PickSelection`](selection::PickSelection) state.
    pub selection: selection::PickSelection,
    #[cfg(feature = "highlight")]
    /// Tracks entity [`PickHighlight`](highlight::PickHighlight) state.
    pub highlight: highlight::PickHighlight,
}

/// Components needed to build a pointer
#[derive(Bundle)]
pub struct PointerBundle {
    /// The pointer's unique [`PointerId`](pointer::PointerId).
    pub id: pointer::PointerId,
    /// Tracks the pointer's location.
    pub location: pointer::PointerLocation,
    /// Tracks the pointer's button press state.
    pub click: pointer::PointerPress,
    /// Tracks the pointer's interaction state.
    pub interaction: output::PointerInteraction,
    #[cfg(feature = "selection")]
    /// Tracks whether the pointer's multiselect is active.
    pub multi_select: selection::PointerMultiselect,
}
impl PointerBundle {
    /// Create a new pointer with the provided `id`.
    pub fn new(id: pointer::PointerId) -> Self {
        PointerBundle {
            id,
            location: pointer::PointerLocation::default(),
            click: pointer::PointerPress::default(),
            interaction: output::PointerInteraction::default(),
            #[cfg(feature = "selection")]
            multi_select: selection::PointerMultiselect::default(),
        }
    }
}

/// Adds default mouse and touch pointers to your app.
pub struct DefaultPointersPlugin;
impl Plugin for DefaultPointersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_default_pointers);
    }
}

/// Spawn default mouse and touch pointers.
pub fn add_default_pointers(mut commands: Commands) {
    commands.spawn_bundle(PointerBundle::new(pointer::PointerId::Mouse));
    // Windows supports up to 20 touch + 10 writing
    for i in 0..30 {
        commands.spawn_bundle(PointerBundle::new(pointer::PointerId::Touch(i)));
    }
}
