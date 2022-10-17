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
//! - How do I add touch support?
//! - Is it possible to do all of this across many windows?
//! - Can I test all of this somehow?
//!
//! These are the problems this crate tries to solve.
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
//! #     fn from_data(event_data: &PointerEventData<E>) -> DeleteMe {
//! #         Self(event_data.target())
//! #     }
//! # }
//! # struct GreetMe(Entity);
//! # impl<E: IsPointerEvent> ForwardedEvent<E> for GreetMe {
//! #     fn from_data(event_data: &PointerEventData<E>) -> GreetMe {
//! #         Self(event_data.target())
//! #     }
//! # }
//! # fn setup(
//! #     mut commands: Commands,
//! # ) {
//! commands
//!     .spawn()
//!     .insert_bundle(PickableBundle::default())       // Make the entity pickable
//!     .insert(PickRaycastTarget::default())           // Marker for the `mod_picking` backend
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

use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy};
use bevy_picking_core::backend::PickingBackend;

// Re-exports
pub use bevy_picking_core::{self as core, backend, focus, output, pointer};

// Optional, feature-gated exports
#[cfg(feature = "highlight")]
pub use bevy_picking_highlight as highlight;
#[cfg(feature = "input")]
pub use bevy_picking_input::{self as input};
#[cfg(feature = "selection")]
pub use bevy_picking_selection as selection;

/// Picking backend exports, feature-gated.
pub mod backends {
    #[cfg(feature = "backend_rapier")]
    pub use bevy_picking_rapier as rapier;
    #[cfg(feature = "backend_raycast")]
    pub use bevy_picking_raycast as raycast;
    #[cfg(feature = "backend_shader")]
    pub use bevy_picking_shader as shader;
    #[cfg(feature = "backend_bevy_ui")]
    pub use bevy_picking_ui as bevy_ui;
}

/// Common imports
pub mod prelude {
    pub use crate::{
        output::{
            EventListenerCommands, ForwardedEvent, IsPointerEvent, PointerClick, PointerDown,
            PointerDrag, PointerDragEnd, PointerDragEnter, PointerDragLeave, PointerDragOver,
            PointerDragStart, PointerDrop, PointerEventData, PointerMove, PointerOut, PointerOver,
            PointerUp,
        },
        pointer::{PointerButton, PointerId, PointerLocation, PointerMap, PointerPress},
        DebugEventsPlugin, DefaultPickingPlugins, PickableBundle,
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

    pub use crate::backends;
    #[cfg(feature = "backend_bevy_ui")]
    pub use crate::backends::bevy_ui::prelude::*;
    #[cfg(feature = "backend_rapier")]
    pub use crate::backends::rapier::prelude::*;
    #[cfg(feature = "backend_raycast")]
    pub use crate::backends::raycast::prelude::*;
    #[cfg(feature = "backend_shader")]
    pub use crate::backends::shader::prelude::*;
}

/// A "batteries-included" set of plugins that adds everything needed for picking, highlighting, and
/// multiselect.
///
/// You will need to add at least one backend to construct this plugin group.
pub struct DefaultPickingPlugins;
impl DefaultPickingPlugins {
    /// Construct a set of picking plugins with the supplied backend.
    pub fn build(backend: impl PickingBackend + 'static) -> DefaultPickingPluginsBuilder {
        let mut result = DefaultPickingPluginsBuilder {
            backends: Vec::new(),
        };
        result.backends.push(Box::new(backend));
        result
    }
}

/// A type that facilitates building picking plugin groups correctly. [`DefaultPickingPlugins`] does
/// not implement [`PluginGroup`], so it cannot be added to a bevy app with `.add_plugins()`. The
/// type [`DefaultPickingPluginsBuilder`] *does* implement `PluginGroups` but can only be created
/// using the `with_backend()` functions. This ensures that when a user is adding this plugin, the
/// type system will guarantee thy have added at least one picking backend.
pub struct DefaultPickingPluginsBuilder {
    backends: Vec<Box<dyn PickingBackend>>,
}

impl DefaultPickingPluginsBuilder {
    /// Adds a backend
    pub fn with_backend(
        mut self,
        backend: impl PickingBackend + 'static,
    ) -> DefaultPickingPluginsBuilder {
        self.backends.push(Box::new(backend));
        self
    }
}

impl PluginGroup for DefaultPickingPluginsBuilder {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(core::CorePlugin)
            .add(core::InteractionPlugin)
            .add(input::InputPlugin);

        // Optional
        #[cfg(feature = "selection")]
        group.add(selection::SelectionPlugin);
        #[cfg(feature = "highlight")]
        highlight::HighlightingPlugins.build(group);

        for mut backend in self.backends.drain(..) {
            backend.build(group);
        }
    }
}

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
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

/// Components needed to build a pointer. Multiple pointers can be active at once, with each pointer
/// being an entity.
///
/// `Mouse` and `Touch` pointers are automatically spawned as needed. Use this bundle if you are
/// spawning a custom `PointerId::Custom` pointer, either for testing, or as a software controlled
/// pointer, or if you are replacing or extending the default touch and mouse inputs.
#[derive(Bundle)]
pub struct PointerBundle {
    #[bundle]
    /// The core pointer components bundle
    pub core: core::PointerCoreBundle,
    #[cfg(feature = "selection")]
    /// Tracks whether the pointer's multiselect is active.
    pub multi_select: selection::PointerMultiselect,
}

impl PointerBundle {
    /// Create a new pointer with the provided [`PointerId`](pointer::PointerId).
    pub fn new(id: pointer::PointerId) -> Self {
        PointerBundle {
            core: core::PointerCoreBundle::new(id),
            #[cfg(feature = "selection")]
            multi_select: selection::PointerMultiselect::default(),
        }
    }
}

/// Logs events for debugging
#[derive(Debug, Default, Clone)]
pub struct DebugEventsPlugin {
    /// Suppresses noisy events like `Move` and `Drag` when set to `false`
    pub noisy: bool,
}
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        let should_run = if self.noisy {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        };

        app.init_resource::<core::debug::Frame>()
            .add_system_to_stage(CoreStage::First, core::debug::increment_frame)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                input::debug::print
                    .before(core::PickStage::Backend)
                    .with_run_criteria(move || should_run),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(core::debug::print::<output::PointerOver>)
                    .with_system(core::debug::print::<output::PointerOut>)
                    .with_system(core::debug::print::<output::PointerDown>)
                    .with_system(core::debug::print::<output::PointerUp>)
                    .with_system(core::debug::print::<output::PointerClick>)
                    .with_system(
                        core::debug::print::<output::PointerMove>
                            .with_run_criteria(move || should_run),
                    )
                    .with_system(core::debug::print::<output::PointerDragStart>)
                    .with_system(
                        core::debug::print::<output::PointerDrag>
                            .with_run_criteria(move || should_run),
                    )
                    .with_system(core::debug::print::<output::PointerDragEnd>)
                    .with_system(core::debug::print::<output::PointerDragEnter>)
                    .with_system(core::debug::print::<output::PointerDragOver>)
                    .with_system(core::debug::print::<output::PointerDragLeave>)
                    .with_system(core::debug::print::<output::PointerDrop>)
                    .label("PointerOutputDebug"),
            );

        #[cfg(feature = "selection")]
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .with_system(core::debug::print::<selection::PointerSelect>)
                .with_system(core::debug::print::<selection::PointerDeselect>),
        );
    }
}
