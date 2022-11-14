//! This exists to allow backends to use things like the picking plugins or bundles.
//!
//! The top level `bevy_mod_picking` library re-exports the backends, which would cause a dependency
//! cycle if the backends also tried to use bundles defined in picking`.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use crate::*;
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_picking_core::backend::PickingBackend;

/// A "batteries-included" set of plugins that adds everything needed for picking, highlighting, and
/// multiselect.
///
/// You will need to add at least one backend to construct this plugin group.
pub struct DefaultPickingPlugins(std::marker::PhantomData<()>);
impl DefaultPickingPlugins {
    /// Create a ndw picking plugin builder
    pub fn start() -> DefaultPickingPluginsBuilder {
        DefaultPickingPluginsBuilder {
            backends: Vec::new(),
        }
    }
}

/// A type that facilitates building picking plugin groups correctly. [`DefaultPickingPlugins`] does
/// not implement [`PluginGroup`], so it cannot be added to a bevy app with `.add_plugins()`. This
/// type *does* implement `PluginGroups` but can only be created using the `with_backend()`
/// functions. This ensures that when a user is adding this plugin, the type system will guarantee
/// thy have added at least one picking backend.
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
    fn build(mut self) -> bevy::app::PluginGroupBuilder {
        let mut builder = bevy::app::PluginGroupBuilder::start::<Self>();

        builder = builder
            .add(core::CorePlugin)
            .add(core::InteractionPlugin)
            .add(input::InputPlugin)
            .add(selection::SelectionPlugin)
            .add(highlight::HighlightingPlugin)
            .add(debug::DebugPickingPlugin::default());

        for backend in self.backends.drain(..) {
            builder = builder.add(backend);
        }

        builder
    }
}

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    /// Tracks entity [`Interaction`] state.
    pub interaction: Interaction,
    /// The entity's configurable [`FocusPolicy`]
    pub focus_policy: FocusPolicy,
    /// Tracks entity [`PickSelection`](selection::PickSelection) state.
    pub selection: selection::PickSelection,
    /// Tracks entity [`PickHighlight`](highlight::PickHighlight) state.
    pub highlight: highlight::PickHighlight,
}
