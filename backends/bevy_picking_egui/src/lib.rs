//! A raycasting backend for [`bevy_egui`]. This backend simply ensures that egui blocks other
//! entities from being picked.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::camera::NormalizedRenderTarget;

use bevy_egui::EguiContext;
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_egui`](crate) crate.
pub mod prelude {
    pub use crate::EguiBackend;
}

/// Adds picking support for [`bevy_egui`], by ensuring that egui blocks other entities from being
/// picked.
#[derive(Clone)]
pub struct EguiBackend;
impl Plugin for EguiBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate, // This is important. If the system is put into the picking set in PreUpdate, the egui frame will not have been constructed, and the backend will not report egui hits, because the user doesn't build egui until the Update schedule. The downside to this is that the backend will always be one frame out of date. The only way to solve this is to do all of your egui work in PreUpdate before the picking backend set, then change this system to run in the picking set.
            egui_picking,
        )
        .insert_resource(EguiBackendSettings::default());

        #[cfg(feature = "selection")]
        app.add_systems(First, update_settings);
    }
}

/// Settings for the [`EguiBackend`].
#[derive(Debug, Default, Resource, Reflect)]
pub struct EguiBackendSettings {
    /// When set to true, clicking on egui will deselect other entities
    #[cfg(feature = "selection")]
    pub allow_deselect: bool,
}

/// Marks the entity used as the pseudo egui pointer.
#[derive(Component, Reflect)]
pub struct EguiPointer;

/// Updates backend to match [`EguiBackendSettings`].
#[cfg(feature = "selection")]
pub fn update_settings(
    mut commands: Commands,
    settings: Res<EguiBackendSettings>,
    egui_context: Query<Entity, With<EguiContext>>,
) {
    if settings.is_added() || settings.is_changed() {
        for entity in &egui_context {
            match settings.allow_deselect {
                true => commands
                    .entity(entity)
                    .remove::<bevy_picking_selection::NoDeselect>(),
                false => commands
                    .entity(entity)
                    .insert(bevy_picking_selection::NoDeselect),
            };
        }
    }
}

/// If egui in the current window is reporting that the pointer is over it, we report a hit.
pub fn egui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    mut egui_context: Query<(Entity, &mut EguiContext)>,
    mut output: EventWriter<PointerHits>,
) {
    for (pointer, location) in pointers
        .iter()
        .filter_map(|(i, p)| p.location.as_ref().map(|l| (i, l)))
    {
        if let NormalizedRenderTarget::Window(id) = location.target {
            if let Ok((entity, mut ctx)) = egui_context.get_mut(id.entity()) {
                if ctx.get_mut().wants_pointer_input() {
                    let entry = (entity, HitData::new(entity, 0.0, None, None));
                    let order = 1_000_000f32; // Assume egui should be on top of everything else.
                    output.send(PointerHits::new(*pointer, Vec::from([entry]), order))
                }
            }
        }
    }
}
