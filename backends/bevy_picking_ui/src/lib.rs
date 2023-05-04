//! A raycasting backend for [`bevy_ui`](bevy::ui).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::ui::{self, FocusPolicy};
use bevy::{prelude::*, render::camera::NormalizedRenderTarget, window::PrimaryWindow};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_ui`](crate) crate.
pub mod prelude {
    pub use crate::BevyUiBackend;
}

/// Adds picking support for [`bevy_ui`](bevy::ui)
#[derive(Clone)]
pub struct BevyUiBackend;
impl PickingBackend for BevyUiBackend {}
impl Plugin for BevyUiBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, ui_picking.in_set(PickSet::Backend));
    }
}

/// Computes the UI node entities under each pointer
pub fn ui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut node_query: Query<
        (
            Entity,
            &ui::Node,
            &GlobalTransform,
            &FocusPolicy,
            Option<&CalculatedClip>,
        ),
        Without<PointerId>,
    >,
    mut output: EventWriter<PointerHits>,
) {
    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location
            .location()
            // TODO: update when proper multi-window UI is implemented
            .filter(|loc| {
                if let NormalizedRenderTarget::Window(window) = loc.target {
                    if primary_window.get(window.entity()).is_ok() {
                        return true;
                    }
                }
                false
            })
            .map(|loc| (pointer, loc))
    }) {
        let camera = cameras
            .iter()
            .find(|(_entity, camera)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            })
            .map(|(entity, _camera)| entity)
            .unwrap_or_else(|| panic!("No camera found associated with pointer {:?}.", pointer));

        let cursor_position = location.position;
        let mut blocked = false;

        let over_list = node_query
            .iter_mut()
            .filter_map(|(entity, node, global_transform, focus, clip)| {
                if blocked {
                    return None;
                }

                blocked = *focus == FocusPolicy::Block;

                let position = global_transform.translation();
                let ui_position = position.truncate();
                let extents = node.size() / 2.0;
                let mut min = ui_position - extents;
                let mut max = ui_position + extents;
                if let Some(clip) = clip {
                    min = min.max(clip.clip.min);
                    max = Vec2::min(max, clip.clip.max);
                }

                let contains_cursor = (min.x..max.x).contains(&cursor_position.x)
                    && (min.y..max.y).contains(&cursor_position.y);

                contains_cursor.then_some((
                    entity,
                    HitData {
                        camera,
                        depth: position.z,
                        position: None,
                        normal: None,
                    },
                ))
            })
            .collect::<Vec<_>>();

        output.send(PointerHits {
            pointer: *pointer,
            picks: over_list,
            order: 10,
        })
    }
}
