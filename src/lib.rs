//! A flexible set of plugins that add picking functionality to your [`bevy`] app, with a focus on
//! ergonomics, expressiveness, and ease of use.
//!
//! # About
//!
//! What is "picking"? Picking is the act of interacting with objects on your screen with a pointer.
//! That pointer might be a mouse cursor, a touch input, or a custom software cursor (such as a game
//! UI cursor controlled with a gamepad). As you make an application interactive, whether it's a
//! traditional 2D UI, or 3D objects, you will run into some recurring challenges:
//!
//! - How do I highlight things?
//! - How can I trigger an event when I click/drag/hover/etc over a thing?
//! - Is it possible to do all of this across many windows?
//! - Will this support multi-touch?
//! - Can I test all of this somehow?
//!
//! These are the problems this crate solves.
//!
//! # Getting Started
//!
//! Making objects pickable is pretty straightforward. In the most minimal cases, it's as simple as:
//!
//! ```
//! # use bevy::prelude::*;
//! use bevy_mod_picking::prelude::*;
//!
//! # struct DeleteMe(Entity);
//! # impl<E: IsPointerEvent> ForwardedEvent<E> for DeleteMe {
//! #     fn from_data(event_data: &EventData<E>) -> DeleteMe {
//! #         Self(event_data.target())
//! #     }
//! # }
//! # struct GreetMe(Entity);
//! # impl<E: IsPointerEvent> ForwardedEvent<E> for GreetMe {
//! #     fn from_data(event_data: &EventData<E>) -> GreetMe {
//! #         Self(event_data.target())
//! #     }
//! # }
//! # fn setup(
//! #     mut commands: Commands,
//! # ) {
//! commands
//!     .spawn((PickableBundle::default(),  // Make the entity pickable
//!         PickRaycastTarget::default()));   // Marker for the `mod_picking` backend
//! # }
//! ```
//!
//! # The Picking Pipeline
//!
//! This plugin is designed to be extremely modular. To do so, it works in well-defined stages that
//! form a pipeline, where events are used to pass data between each stage. All the types needed for
//! the pipeline are defined in the [`bevy_picking_core`] crate.
//!
//! ## Input ([`bevy_picking_input`])
//!
//! The first stage of the pipeline is to gather inputs and create pointers. This stage is
//! ultimately responsible for generating [`InputMove`](bevy_picking_core::pointer::InputMove) and
//! [`InputPress`](bevy_picking_core::pointer::InputPress) events. The provided crate does this
//! automatically for mouse, touch, and pen inputs. If you wanted to implement your own pointer,
//! controlled by some other input, you can do that here.
//!
//! Because pointer positions and presses are driven by these events, you can use them to mock
//! inputs for testing.
//!
//! ## Backend ([`bevy_picking_core::backend`])
//!
//! The job of a backend is extremely simple: given the location of a pointer (from the input
//! stage), return a list of all entities under that pointer.
//!
//! You will eventually need to choose which picking backend(s) you want to use. This plugin uses
//! `bevy_mod_raycast` by default; it works with bevy `Mesh`es out of the box and requires no extra
//! dependencies. These qualities make it useful when prototyping, however it is not particularly
//! performant for large meshes. You can consider switching to the rapier backends if performance
//! becomes a problem. For simple or low-poly games, it may never be an issue.
//!
//! However, it's important to understand that you can mix and match backends! This crate provides
//! some backends out of the box, but you can even write your own. It's been made as easy as
//! possible intentionally; the entire `bevy_mod_raycast` backend is less than 100 lines of code.
//! For example, you might have a backend for your UI, and one for the 3d scene, with each being
//! specialized for their purpose.
//!
//! ## Focus ([`bevy_picking_core::focus`])
//!
//! The final step is to use the data from the previous stages, combine and sort the results, and
//! determine what each cursor is hovering over. With this information, high-level pointer events
//! are generated, such as events that trigger when a pointer enters an entity, or when a pointer
//! drags and drops one entity onto another.
//!
//! From here, these high-level events are used for highlighting, selection, event bubbling, and all
//! the other features this crate provides. Because it is completely agnostic to the the earlier
//! stages of the pipeline, you can easily extend the plugin with arbitrary backends and input
//! methods.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_picking_core::PointerCoreBundle;

use bevy::prelude::Bundle;
pub use bevy_picking_core::{self as core, backend, focus, output, pointer};
pub use bevy_picking_input::{self as input};

#[cfg(feature = "highlight")]
pub use bevy_picking_highlight as highlight;
#[cfg(feature = "selection")]
pub use bevy_picking_selection as selection;
#[cfg(feature = "debug")]
pub mod debug;

pub mod plugins;

/// Picking backend exports, feature-gated.
pub mod backends {
    #[cfg(feature = "backend_egui")]
    pub use bevy_picking_egui as egui;
    #[cfg(feature = "backend_rapier")]
    pub use bevy_picking_rapier as rapier;
    #[cfg(feature = "backend_raycast")]
    pub use bevy_picking_raycast as raycast;
    #[cfg(feature = "backend_shader")]
    pub use bevy_picking_shader as shader;
    #[cfg(feature = "backend_sprite")]
    pub use bevy_picking_sprite as sprite;
    #[cfg(feature = "backend_bevy_ui")]
    pub use bevy_picking_ui as bevy_ui;
}

/// Common imports
pub mod prelude {
    #[cfg(feature = "debug")]
    pub use crate::debug::DebugPickingPlugin;
    pub use crate::{
        backends,
        output::{
            EventData, EventListenerCommands, ForwardedEvent, IsPointerEvent, PointerClick,
            PointerDown, PointerDrag, PointerDragEnd, PointerDragEnter, PointerDragLeave,
            PointerDragOver, PointerDragStart, PointerDrop, PointerMove, PointerOut, PointerOver,
            PointerUp,
        },
        plugins::{DefaultPickingPlugins, PickableBundle},
        pointer::{PointerButton, PointerId, PointerLocation, PointerMap, PointerPress},
        *,
    };

    #[cfg(feature = "highlight")]
    pub use crate::highlight::{
        CustomHighlightPlugin, DefaultHighlighting, HighlightOverride, HighlightingPlugin,
        PickHighlight,
    };

    #[cfg(feature = "selection")]
    pub use crate::selection::{
        NoDeselect, PickSelection, PointerDeselect, PointerMultiselect, PointerSelect,
        SelectionPlugin,
    };

    #[cfg(feature = "backend_bevy_ui")]
    pub use backends::bevy_ui::prelude::*;
    #[cfg(feature = "backend_egui")]
    pub use backends::egui::prelude::*;
    #[cfg(feature = "backend_rapier")]
    pub use backends::rapier::prelude::*;
    #[cfg(feature = "backend_raycast")]
    pub use backends::raycast::prelude::*;
    #[cfg(feature = "backend_shader")]
    pub use backends::shader::prelude::*;
    #[cfg(feature = "backend_sprite")]
    pub use backends::sprite::prelude::*;
}

/// Bundle of components needed for a fully-featured pointer.
#[derive(Bundle)]
pub struct PointerBundle {
    #[bundle]
    core: PointerCoreBundle,
    #[cfg(feature = "selection")]
    selection: selection::PointerMultiselect,
}
impl PointerBundle {
    /// Build a new `PointerBundle` with the supplied [`PointerId`].
    pub fn new(id: pointer::PointerId) -> PointerBundle {
        PointerBundle {
            core: PointerCoreBundle::new(id),
            #[cfg(feature = "selection")]
            selection: selection::PointerMultiselect::default(),
        }
    }
}
