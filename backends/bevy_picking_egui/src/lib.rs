//! A raycasting backend for [`bevy_egui`]

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, render::camera::NormalizedRenderTarget};
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
impl PickingBackend for EguiBackend {}
impl Plugin for EguiBackend {
    fn build(&self, app: &mut App) {
        app.add_system(egui_picking.in_set(PickSet::Backend));
    }
}

/// Marks the entity used as the pseudo egui pointer.
#[derive(Component, Reflect)]
pub struct EguiPointer;

/// If egui in the current window is reporting that the pointer is over it, we report that with a
/// pick depth of -1, so it is on top of all other entities.
pub fn egui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    mut egui_context: Query<(Entity, &mut EguiContext)>,
    mut output: EventWriter<EntitiesUnderPointer>,
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
                        PickData {
                            depth: -1.0,
                            position: None,
                            normal: None,
                        },
                    );

                    output.send(EntitiesUnderPointer {
                        pointer: *pointer,
                        picks: Vec::from([entry]),
                    })
                }
            }
        }
    }
}
