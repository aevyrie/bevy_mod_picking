//! A raycasting backend for [`bevy_egui`]

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, render::camera::RenderTarget};
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
        app.add_startup_system(spawn_egui_entity)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .label(PickStage::Backend)
                    .with_system(egui_picking),
            );
    }
}

/// Used to track an entity ID for egui, so the egui picking backend can report that there is some
/// entity under the pointer.
#[derive(Resource, Reflect)]
pub struct EguiEntity(Entity);

/// Marks the entity used as the pseudo egui pointer.
#[derive(Component, Reflect)]
pub struct EguiPointer;

fn spawn_egui_entity(mut commands: Commands) {
    let id = commands
        .spawn((
            EguiPointer,
            #[cfg(feature = "selection")]
            bevy_picking_selection::NoDeselect,
            Name::new("egui"),
        ))
        .id();
    commands.insert_resource(EguiEntity(id));
}

/// If egui in the current window is reporting that the pointer is over it, we report that with a
/// pick depth of -1, so it is on top of all other entities.
pub fn egui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    egui_entity: Res<EguiEntity>,
    egui_context: Option<ResMut<EguiContext>>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    let mut egui = match egui_context {
        Some(c) => c,
        None => return,
    };

    for (pointer, location) in pointers
        .iter()
        .filter_map(|(i, p)| p.location.as_ref().map(|l| (i, l)))
    {
        if let RenderTarget::Window(id) = location.target {
            if let Some(ctx) = egui.try_ctx_for_window_mut(id) {
                if ctx.is_pointer_over_area() {
                    let entity = EntityDepth {
                        entity: egui_entity.0,
                        depth: -1.0,
                    };

                    output.send(EntitiesUnderPointer {
                        pointer: *pointer,
                        over_list: Vec::from([entity]),
                    })
                }
            }
        }
    }
}
