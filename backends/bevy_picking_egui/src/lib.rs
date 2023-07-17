//! A raycasting backend for [`bevy_egui`]. This backend simply ensures that egui blocks other
//! entities from being picked.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, render::camera::NormalizedRenderTarget};
use bevy_egui::{EguiContext, EguiSet};
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
            Update,
            egui_picking
                .in_set(PickSet::Backend)
                .after(EguiSet::BeginFrame),
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
                if ctx.get_mut().is_pointer_over_area() {
                    let entry = (
                        entity,
                        HitData {
                            camera: entity,
                            depth: 0.0,
                            position: None,
                            normal: None,
                        },
                    );

                    output.send(PointerHits {
                        pointer: *pointer,
                        picks: Vec::from([entry]),
                        order: 1_000_000, // Assume egui should be on top of everything else.
                    })
                }
            }
        }
    }
}
