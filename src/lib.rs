//! A flexible set of plugins that add picking functionality to your [`bevy`] app, with a focus on
//! ergonomics, modularity, and ease of use.
//!
//! #### Lightweight
//!
//! Only pay for what you use. All non-critical plugins can be disabled, including highlighting,
//! selection, and any backends not in use.
//!
//! #### Expressive
//!
//! This plugin produces convenient [`PointerEvent`]s similar to those available in JS on the web:
//! [`Click`], [`Over`], [`Drag`], and many more. To make it even easier to handle events that
//! target a single entity, the [`EventListener`] component and event bubbling are provided. This
//! allows you to run callbacks or forward events when, for example, the child of an entity is
//! clicked on. This makes it very easy to express:
//!
//! > If entity X is hovered/clicked/dragged/etc, do Y
//!
//! #### Modular Backends
//!
//! Picking backends run hit tests to determine if a pointer is over any entities. This plugin
//! provides a simple API to write your own backend in ~100 lines of code and includes half a dozen
//! out of the box. These include backends for `rapier`, `bevy_mod_raycast`, and `bevy_egui` among
//! others. Multiple backends can be used at the same time!
//!
//! #### Input Agnostic
//!
//! Pointers can be controlled with anything, whether its the included mouse or touch inputs, or a
//! custom gamepad input system you write yourself.
//!
//! #### Robust
//!
//! In addition to these features, this plugin also correctly handles multitouch and multiple
//! windows. Some backends, like the one for `bevy_ui`, cannot support multiple windows due to
//! limitations of the plugin they are built for.
//!
//! # Getting Started
//!
//! Making objects pickable is pretty straightforward. In the most minimal cases, it's as simple as:
//!
//! ```
//! # use bevy::prelude::*;
//! use bevy_mod_picking::prelude::*;
//! # fn setup(
//! #     mut commands: Commands,
//! #     app: &mut App,
//! # ) {
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(DefaultPickingPlugins);
//!
//! commands.spawn((
//!     PbrBundle::default(),           // The `bevy_picking_raycast` backend works with meshes
//!     PickableBundle::default(),      // Makes the entity pickable
//!     RaycastPickTarget::default()    // Marker for the `bevy_picking_raycast` backend
//! ));
//!
//! commands.spawn((
//!     Camera3dBundle::default(),
//!     RaycastPickCamera::default(),   // Enable picking using this camera
//! ));
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
//! The next step is to use the data from the previous stages, combine and sort the results, and
//! determine what each cursor is hovering over.
//!
//! ### Events ([`bevy_picking_core::events`])
//!
//! In the final step, the high-level pointer events are generated, such as events that trigger when
//! a pointer hovers or clicks an entity. These simple events are then used to generate more complex
//! events for dragging and dropping. Once all events have been generated, the event bubbline
//!
//!  Because it is completely agnostic to the the earlier stages of the pipeline, you can easily
//! extend the plugin with arbitrary backends and input methods.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::Bundle, ui::Interaction};
use bevy_picking_core::PointerCoreBundle;
use prelude::*;

pub use bevy_picking_core::{self as core, backend, events, focus, pointer};
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
        events::{
            Bubble, Click, Down, Drag, DragEnd, DragEnter, DragLeave, DragOver, DragStart, Drop,
            EventListener, EventListenerCommands, EventListenerData, ForwardedEvent,
            IsPointerEvent, Move, Out, Over, PointerEvent, Up,
        },
        plugins::DefaultPickingPlugins,
        pointer::{PointerButton, PointerId, PointerLocation, PointerMap, PointerPress},
        *,
    };

    #[cfg(feature = "highlight")]
    pub use crate::highlight::{
        DefaultHighlightingPlugin, GlobalHighlight, Highlight, HighlightKind, HighlightPlugin,
        PickHighlight,
    };

    #[cfg(feature = "selection")]
    pub use crate::selection::{
        Deselect, NoDeselect, PickSelection, PointerMultiselect, Select, SelectionPlugin,
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

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    /// Tracks entity [`Interaction`] state.
    pub interaction: Interaction,
    /// Tracks entity [`PickSelection`](selection::PickSelection) state.
    #[cfg(feature = "selection")]
    pub selection: selection::PickSelection,
    /// Tracks entity [`PickHighlight`](highlight::PickHighlight) state.
    #[cfg(feature = "highlight")]
    pub highlight: highlight::PickHighlight,
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
    pub fn new(id: PointerId) -> PointerBundle {
        PointerBundle {
            core: PointerCoreBundle::new(id),
            #[cfg(feature = "selection")]
            selection: selection::PointerMultiselect::default(),
        }
    }
}

/// Used for examples to reduce picking latency. Not relevant code for the examples.
#[allow(dead_code)]
pub fn low_latency_window_plugin() -> bevy::window::WindowPlugin {
    bevy::window::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            present_mode: bevy::window::PresentMode::Mailbox,
            ..Default::default()
        }),
        ..Default::default()
    }
}
