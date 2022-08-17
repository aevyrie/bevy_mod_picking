//! A raycasting backend for [`bevy_ui`].

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, render::camera::RenderTarget, window::WindowId};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_ui`] crate.
pub mod prelude {
    pub use crate::UiPickingPlugin;
}

/// Adds picking support for [`bevy_ui`](bevy::ui)
pub struct UiPickingPlugin;
impl Plugin for UiPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .label(PickStage::Backend)
                .with_system(ui_picking),
        );
    }
}

/// Computes the UI node entities under each pointer
pub fn ui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    mut node_query: Query<(Entity, &Node, &GlobalTransform, Option<&CalculatedClip>)>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    for (pointer, position) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location
            .location()
            // TODO: update when proper multi-window UI is implemented
            .filter(|loc| loc.target == RenderTarget::Window(WindowId::primary()))
            .map(|loc| (pointer, loc.position))
    }) {
        let cursor_position = position;
        let over_list = node_query
            .iter_mut()
            .filter_map(|(entity, node, global_transform, clip)| {
                let position = global_transform.translation();
                let ui_position = position.truncate();
                let extents = node.size / 2.0;
                let mut min = ui_position - extents;
                let mut max = ui_position + extents;
                if let Some(clip) = clip {
                    min = Vec2::max(min, clip.clip.min);
                    max = Vec2::min(max, clip.clip.max);
                }

                let contains_cursor = (min.x..max.x).contains(&cursor_position.x)
                    && (min.y..max.y).contains(&cursor_position.y);

                if contains_cursor {
                    Some(EntityDepth {
                        entity,
                        depth: position.z,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        output.send(EntitiesUnderPointer {
            id: *pointer,
            over_list,
        })
    }
}
