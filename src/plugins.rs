//! This exists to allow backends to use things like the picking plugins or bundles.
//!
//! The top level `bevy_mod_picking` library re-exports the backends, which would cause a dependency
//! cycle if the backends also tried to use bundles defined in picking`.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use crate::*;
use bevy::prelude::*;

/// A "batteries-included" set of plugins that adds everything needed for picking, highlighting, and
/// multiselect. Backends are automatically added if their corresponding feature is enabled.
pub struct DefaultPickingPlugins;

impl PluginGroup for DefaultPickingPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut builder = bevy::app::PluginGroupBuilder::start::<Self>();

        builder = builder
            .add(core::CorePlugin)
            .add(core::InteractionPlugin)
            .add(input::InputPlugin);

        #[cfg(feature = "debug")]
        {
            builder = builder.add(debug::DebugPickingPlugin::default());
        }

        #[cfg(feature = "highlight")]
        {
            builder = builder.add(highlight::HighlightingPlugin);
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
