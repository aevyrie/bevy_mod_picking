//! A flexible set of plugins that add picking functionality to your [`bevy`] app, with a focus on
//! modularity, expressiveness, and robustness. Want to drag a UI entity and drop it onto a 3D mesh
//! entity? This plugin allows you to add event listeners to **any** entity, and works with mouse,
//! touch, or even gamepads. Includes optional integrations for `rapier` and `egui`, but is agnostic
//! to the picking backend.
//!
//! #### Lightweight
//!
//! Only compile what you use. All non-critical plugins can be disabled, including highlighting,
//! selection, and any backends not in use.
//!
//! #### Expressive
//!
//! [`Pointer`] events make it easy to react to interactions like [`Click`], [`Over`], or [`Drag`]
//! (13 pointer events are provided). Reacting to these interaction events on a specific entity is
//! made possible with the [`On<Event>`] component. When events are generated, they bubble up the
//! entity hierarchy starting from their target, looking for these event listener components.
//!
//! This allows you to run callbacks when any children of an entity are interacted with:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy::ecs::system::Command;
//! # use prelude::*;
//! # use bevy_mod_picking::prelude::*;
//! # use bevy_eventlistener::callbacks::ListenerInput;
//! #
//! # struct DeleteTarget;
//! # impl From<ListenerInput<Pointer<Click>>> for DeleteTarget {
//! #     fn from(_: ListenerInput<Pointer<Click>>) -> Self {
//! #         DeleteTarget
//! #     }
//! # }
//! # impl Command for DeleteTarget {
//! #     fn apply(self, world: &mut World) {}
//! # }
//! #
//! # #[derive(Event)]
//! # struct Greeting;
//! # impl From<ListenerInput<Pointer<Over>>> for Greeting {
//! #     fn from(_: ListenerInput<Pointer<Over>>) -> Self {
//! #         Greeting
//! #     }
//! # }
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         // Spawn your entity here, e.g. a Mesh.
//!         // When dragged, mutate the `Transform` component on the dragged target entity:
//!         On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
//!             transform.rotate_local_y(drag.delta.x / 50.0)
//!         }),
//!         On::<Pointer<Click>>::add_command::<DeleteTarget>(),
//!         On::<Pointer<Over>>::send_event::<Greeting>(),
//!     ));
//! }
//! ```
//!
//! #### Modular
//!
//! Picking backends run hit tests to determine if a pointer is over any entities. This plugin
//! provides a [simple API to write your own backend](crate::backend) in about 100 lines of code; it
//! also includes half a dozen backends out of the box. These include `rapier`, `bevy_mod_raycast`,
//! and `bevy_egui` among others. Multiple backends can be used at the same time!
//!
//! #### Input Agnostic
//!
//! Pointers can be controlled with anything, whether its the included mouse or touch inputs, or a
//! custom gamepad input system you write yourself.
//!
//! #### Robust
//!
//! In addition to these features, this plugin also correctly handles multitouch and multiple
//! windows.
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
//! #### Next Steps
//!
//! To learn more, take a look at the examples in the
//! [`./examples`](https://github.com/aevyrie/bevy_mod_picking/tree/main/examples) directory.
//!
//! # The Picking Pipeline
//!
//! This plugin is designed to be extremely modular. To do so, it works in well-defined stages that
//! form a pipeline, where events are used to pass data between each stage. All the types needed for
//! the pipeline are defined in the [`bevy_picking_core`] crate.
//!
//! #### Input ([`bevy_picking_input`])
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
//! After inputs are generated, they are then collected to update the current [`PointerLocation`]
//! for each pointer.
//!
//! #### Backend ([`bevy_picking_core::backend`])
//!
//! A picking backend only has one job: reading [`PointerLocation`](crate::pointer::PointerLocation)
//! components, and producing [`PointerHits`](crate::backend::PointerHits).
//!
//! You will eventually need to choose which picking backend(s) you want to use. This plugin uses
//! `bevy_mod_raycast` by default; it works with bevy `Mesh`es out of the box and requires no extra
//! dependencies. These qualities make it useful when prototyping, however it is not particularly
//! performant for large meshes. Consider switching to the rapier backend if performance becomes a
//! problem or if you already have the dependency in-tree. For simple or low-poly games, it may
//! never be an issue.
//!
//! It's important to understand that you can mix and match backends! For example, you might have a
//! backend for your UI, and one for the 3d scene, with each being specialized for their purpose.
//! This crate provides some backends out of the box, but you can even write your own. It's been
//! made as easy as possible intentionally; the entire `bevy_mod_raycast` backend is less than 100
//! lines of code.
//!
//!
//! #### Focus ([`bevy_picking_core::focus`])
//!
//! The next step is to use the data from the backends, combine and sort the results, and determine
//! what each cursor is hovering over, producing a [`HoverMap`](`crate::focus::HoverMap`). Note that
//! just because a pointer is over an entity, it is not necessarily hovering that entity. Although
//! multiple backends may be reporting that a pointer is over an entity, the focus system needs to
//! determine which one(s) are actually being hovered based on the pick depth, order of the backend,
//! and the [`FocusPolicy`](bevy::ui::FocusPolicy) of the entity. In other words, if one entity is
//! in front of another, only the topmost one will be hovered, even if the pointer is within the
//! bounds of both entities.
//!
//! #### Events ([`bevy_picking_core::events`])
//!
//! In the final step, the high-level pointer events are generated, such as events that trigger when
//! a pointer hovers or clicks an entity. These simple events are then used to generate more complex
//! events for dragging and dropping. Once all events have been generated, the event bubbling
//! systems propagate events through the entity hierarchy, triggering [`On<E>`] callbacks.
//!
//! Because it is completely agnostic to the the earlier stages of the pipeline, you can easily
//! extend the plugin with arbitrary backends and input methods.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::Bundle, ui::Interaction};
use bevy_picking_core::PointerCoreBundle;
use prelude::*;

pub use bevy_picking_core::{self as picking_core, backend, events, focus, pointer};
pub use bevy_picking_input::{self as input};

#[cfg(feature = "highlight")]
pub use bevy_picking_highlight as highlight;
#[cfg(feature = "selection")]
pub use bevy_picking_selection as selection;
#[cfg(feature = "debug")]
pub mod debug;

/// Picking backend exports, feature-gated.
pub mod backends {
    #[cfg(feature = "backend_egui")]
    pub use bevy_picking_egui as egui;
    #[cfg(feature = "backend_rapier")]
    pub use bevy_picking_rapier as rapier;
    #[cfg(feature = "backend_raycast")]
    pub use bevy_picking_raycast as raycast;
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
            Click, Down, Drag, DragEnd, DragEnter, DragLeave, DragOver, DragStart, Drop,
            IsPointerEvent, Move, Out, Over, Pointer, Up,
        },
        picking_core::Pickable,
        pointer::{PointerButton, PointerId, PointerLocation, PointerMap, PointerPress},
        *,
    };

    pub use bevy_eventlistener::prelude::*;

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
    /// Marks pickable entities.
    pub pickable: Pickable,
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

/// A "batteries-included" set of plugins that adds everything needed for picking, highlighting, and
/// multiselect. Backends are automatically added if their corresponding feature is enabled.
pub struct DefaultPickingPlugins;

impl bevy::prelude::PluginGroup for DefaultPickingPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut builder = bevy::app::PluginGroupBuilder::start::<Self>();

        builder = builder
            .add(picking_core::CorePlugin)
            .add(picking_core::InteractionPlugin)
            .add(input::InputPlugin);

        #[cfg(feature = "debug")]
        {
            builder = builder.add(debug::DebugPickingPlugin::default());
        }

        #[cfg(feature = "highlight")]
        {
            builder = builder.add(highlight::DefaultHighlightingPlugin);
        }

        #[cfg(feature = "selection")]
        {
            builder = builder.add(selection::SelectionPlugin);
        }

        #[cfg(feature = "backend_raycast")]
        {
            builder = builder.add(bevy_picking_raycast::RaycastBackend);
        }
        #[cfg(feature = "backend_bevy_ui")]
        {
            builder = builder.add(bevy_picking_ui::BevyUiBackend);
        }
        #[cfg(feature = "backend_rapier")]
        {
            builder = builder.add(bevy_picking_rapier::RapierBackend);
        }
        #[cfg(feature = "backend_shader")]
        {
            builder = builder.add(bevy_picking_shader::ShaderBackend);
        }
        #[cfg(feature = "backend_sprite")]
        {
            builder = builder.add(bevy_picking_sprite::SpriteBackend);
        }
        #[cfg(feature = "backend_egui")]
        {
            builder = builder.add(bevy_picking_egui::EguiBackend);
        }

        builder
    }
}

/// Used for examples to reduce picking latency. Not relevant code for the examples.
#[doc(hidden)]
#[allow(dead_code)]
pub fn low_latency_window_plugin() -> bevy::window::WindowPlugin {
    bevy::window::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default()
    }
}
