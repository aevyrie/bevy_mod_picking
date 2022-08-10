//! A raycasting backend for [`bevy_ui`].

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_picking_core::{backend::prelude::*, pointer::Location};

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

// TODO: update when proper multi-window UI is implemented

/// Computes the UI node entities under each pointer
pub fn ui_picking(
    pointers: Query<(Entity, &PointerLocation)>,
    camera: Query<(&Camera, Option<&UiCameraConfig>)>,
    windows: Res<Windows>,
    mut node_query: Query<(Entity, &Node, &GlobalTransform, Option<&CalculatedClip>)>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    for (pointer, Location { target, position }) in
        pointers.iter().filter_map(|(pointer, pointer_location)| {
            pointer_location.location().map(|loc| (pointer, loc))
        })
    {
        // let mut moused_over_z_sorted_nodes = node_query
        //     .iter_mut()
        //     .filter_map(|(entity, node, global_transform, clip)| {
        //         let position = global_transform.translation();
        //         let ui_position = position.truncate();
        //         let extents = node.size / 2.0;
        //         let mut min = ui_position - extents;
        //         let mut max = ui_position + extents;
        //         if let Some(clip) = clip {
        //             min = Vec2::max(min, clip.clip.min);
        //             max = Vec2::min(max, clip.clip.max);
        //         }

        //         let contains_cursor = if let Some(cursor_position) = cursor_position {
        //             (min.x..max.x).contains(&cursor_position.x)
        //                 && (min.y..max.y).contains(&cursor_position.y)
        //         } else {
        //             false
        //         };

        //         if contains_cursor {
        //             Some((entity, focus_policy, interaction, FloatOrd(position.z)))
        //         } else {
        //             if let Some(mut interaction) = interaction {
        //                 if *interaction == Interaction::Hovered
        //                     || (cursor_position.is_none() && *interaction != Interaction::None)
        //                 {
        //                     *interaction = Interaction::None;
        //                 }
        //             }
        //             None
        //         }
        //     })
        //     .collect::<Vec<_>>();
    }
}
