//! A flexible set of plugins that add picking functionality to your `bevy` app.
//!
//! ## Overview
//!
//! In the simplest case, this plugin allows you to click on things in the scene. However, it also
//! allows you to express more complex interactions, like detecting when a touch input drags a UI
//! element and drops it on a 3d mesh rendered to a different camera. The crate also provides event
//! listeners, so you can attach `On<Click>` components to an entity, to run a one-shot bevy system.
//!
//! The plugin works with any input, including mouse, touch, pens, or virtual pointers controlled by
//! gamepads. It includes (optional) backends for `rapier`, `bevy_xpbd`, `bevy_mod_raycast`,
//! `bevy_ui`, `bevy_sprite`, and `egui`, that can be mixed and matched out of the box, or you can
//! write your own.
//!
//! At its core, this crate provides a robust abstraction for computing picking state regardless of
//! pointing devices, or what you are hit testing against.
//!
//! ## Expressive Events
//!
//! The plugin provides normal bevy events that can be listened to with `EventReader`s. These
//! [`Pointer`] events allow you to respond to interactions like [`Click`], [`Over`], or [`Drag`]
//! (13 pointer events are provided). However, this often leads to a lot of boilerplate when you try
//! to do something in response to that click, and you want the behavior to be tied to the entity
//! being clicked on.
//!
//! Reacting to these interaction events on a specific entity is made possible with the
//! [`On<Event>`](On) component. When events are generated, they bubble up the entity hierarchy
//! starting from their target, looking for these event listener components. (See
//! [`bevy_eventlistener`] for details.)
//!
//! This allows you to run callbacks when any children of an entity are interacted with, and leads
//! to succinct, expressive code:
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
//! If you don't need event bubbling or callbacks, you can respond to pointer events like you would
//! any other bevy event, using `EventReader<Pointer<Click>>`, `EventReader<Pointer<Move>>`, etc.
//!
//! ## Modularity
//!
//! #### Mix and Match Hit Testing Backends
//!
//! The plugin attempts to handle all the hard parts for you, all you need to do is tell it when a
//! pointer is hitting any entities. Multiple backends can be used at the same time!
//!
//! [Use this simple API to write your own backend](crate::backend) in about 100 lines of code. The
//! plugin also includes half a dozen backends out of the box. These include `rapier`, `bevy_xpbd`,
//! `bevy_mod_raycast`, `bevy_ui`, `bevy_egui`, and `bevy_sprite`.
//!
//! #### Input Agnostic
//!
//! Pointers can be controlled with anything, whether its the included mouse or touch inputs, or a
//! custom gamepad input system you write yourself.
//!
//! ## Robustness
//!
//! In addition to these features, this plugin also correctly handles multitouch, multiple windows,
//! multiple cameras, viewports, and render layers. Using this as a library allows you to write a
//! picking backend that can interoperate with any other picking backend.
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
//!     .add_plugins(DefaultPickingPlugins); // Includes a mesh raycasting backend by default
//!
//! commands.spawn((
//!     PbrBundle::default(),           // The raycasting backend works with meshes
//!     PickableBundle::default(),      // Makes the entity pickable, and adds optional features
//! ));
//!
//! commands.spawn(Camera3dBundle::default());
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
//! A picking backend only has one job: reading [`PointerLocation`] components, and producing
//! [`PointerHits`](crate::backend::PointerHits).
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
//! and the [`Pickable`] state of the entity. In other words, if one entity is in front of another,
//! only the topmost one will be hovered, even if the pointer is within the bounds of both entities.
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

use bevy_ecs::prelude::*;
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
    #[cfg(feature = "backend_xpbd")]
    pub use bevy_picking_xpbd as xpbd;
}

/// Common imports
pub mod prelude {
    #[cfg(feature = "debug")]
    pub use crate::debug::{DebugPickingMode, DebugPickingPlugin};
    pub use crate::{
        backends,
        events::{
            Click, Down, Drag, DragEnd, DragEnter, DragLeave, DragOver, DragStart, Drop, Move, Out,
            Over, Pointer, Up,
        },
        focus::PickingInteraction,
        input::prelude::*,
        picking_core::Pickable,
        pointer::{
            PointerButton, PointerId, PointerInteraction, PointerLocation, PointerMap, PointerPress,
        },
        *,
    };

    pub use bevy_eventlistener::prelude::*;

    #[cfg(feature = "highlight")]
    pub use crate::highlight::prelude::*;

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
    #[cfg(feature = "backend_xpbd")]
    pub use backends::xpbd::prelude::*;
}

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    /// Provides overrides for picking behavior.
    pub pickable: Pickable,
    /// Tracks entity interaction state.
    pub interaction: focus::PickingInteraction,
    /// Tracks entity [`PickSelection`] state.
    #[cfg(feature = "selection")]
    pub selection: selection::PickSelection,
    /// Tracks entity [`PickHighlight`] state.
    #[cfg(feature = "highlight")]
    pub highlight: highlight::PickHighlight,
}

/// Bundle of components needed for a fully-featured pointer.
#[derive(Bundle)]
pub struct PointerBundle {
    core: PointerCoreBundle,
    #[allow(missing_docs)]
    #[cfg(feature = "selection")]
    pub selection: selection::PointerMultiselect,
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

impl bevy_app::PluginGroup for DefaultPickingPlugins {
    fn build(self) -> bevy_app::PluginGroupBuilder {
        let mut builder = bevy_app::PluginGroupBuilder::start::<Self>();

        builder = builder
            .add(picking_core::CorePlugin)
            .add(picking_core::InteractionPlugin)
            .add(input::InputPlugin);

        #[cfg(feature = "debug")]
        {
            builder = builder.add(debug::DebugPickingPlugin);
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
        #[cfg(feature = "backend_xpbd")]
        {
            builder = builder.add(bevy_picking_xpbd::XpbdBackend);
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
pub fn low_latency_window_plugin() -> bevy_window::WindowPlugin {
    bevy_window::WindowPlugin {
        primary_window: Some(bevy_window::Window {
            present_mode: bevy_window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default()
    }
}
