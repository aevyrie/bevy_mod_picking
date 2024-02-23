//! A picking backend for [`bevy_ui`].
//!
//! # Usage
//!
//! This backend does not require markers on cameras or entities to function. It will look for any
//! pointers using the same render target as the UI camera, and run hit tests on the UI node tree.
//!
//! ## Important Note
//!
//! This backend completely ignores [`FocusPolicy`](bevy_ui::FocusPolicy). The design of bevy ui's
//! focus systems and the picking plugin are not compatible. Instead, use the [`Pickable`] component
//! to customize how an entity responds to picking focus.
//!
//! ## Implementation Notes
//!
//! - Bevy ui can only render to the primary window
//! - Bevy ui can render on any camera with a flag, it is special, and is not tied to a particular
//!   camera.
//! - To correctly sort picks, the order of bevy UI is set to be the camera order plus 0.5.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, query::WorldQuery};
use bevy_render::{camera::NormalizedRenderTarget, prelude::*};
use bevy_transform::prelude::*;
use bevy_ui::{prelude::*, RelativeCursorPosition, UiStack};
use bevy_window::PrimaryWindow;

use bevy_picking_core::backend::prelude::*;
use bevy_picking_core::pointer::Location;

/// Commonly used imports for the [`bevy_picking_ui`](crate) crate.
pub mod prelude {
    pub use crate::BevyUiBackend;
}

/// Adds picking support for [`bevy_ui`].
#[derive(Clone)]
pub struct BevyUiBackend;
impl Plugin for BevyUiBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, ui_picking.in_set(PickSet::Backend));
    }
}

/// Main query from bevy's `ui_focus_system`
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct NodeQuery {
    entity: Entity,
    node: &'static Node,
    global_transform: &'static GlobalTransform,
    relative_cursor_position: Option<&'static mut RelativeCursorPosition>,
    pickable: Option<&'static Pickable>,
    calculated_clip: Option<&'static CalculatedClip>,
    view_visibility: Option<&'static ViewVisibility>,
}

/// Computes the UI node entities under each pointer.
///
/// Bevy's [`UiStack`] orders all nodes in the order they will be rendered, which is the same order
/// we need for determining picking.
pub fn ui_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, Option<&UiCameraConfig>)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    ui_stack: Res<UiStack>,
    ui_scale: Option<Res<UiScale>>,
    mut node_query: Query<NodeQuery>,
    mut output: EventWriter<PointerHits>,
) {
    let ui_scale = ui_scale.map(|f| f.0).unwrap_or(1.0) as f32;
    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location
            .location()
            // TODO: update when proper multi-window UI is implemented
            .filter(|loc| {
                if let NormalizedRenderTarget::Window(window) = loc.target {
                    if primary_window.contains(window.entity()) {
                        return true;
                    }
                }
                false
            })
            .cloned()
            .map(|loc| {
                (
                    pointer,
                    Location {
                        position: loc.position / ui_scale,
                        ..loc
                    },
                )
            })
    }) {
        let window_entity = primary_window.single();

        // Find the topmost bevy_ui camera with the same target as this pointer.
        //
        // Bevy ui can render on many cameras, but it will be the same UI, and we only want to
        // consider the topmost one rendering UI in this window.
        let mut ui_cameras: Vec<_> = cameras
            .iter()
            .filter(|(_entity, camera, _)| {
                camera.is_active
                    && camera.target.normalize(Some(window_entity)).unwrap() == location.target
            })
            .filter(|(_, _, ui_config)| ui_config.map(|config| config.show_ui).unwrap_or(true))
            .collect();
        ui_cameras.sort_by_key(|(_, camera, _)| camera.order);

        // The last camera in the list will be the one with the highest order, and be the topmost.
        let Some((camera_entity, camera, _)) = ui_cameras.last() else {
            continue;
        };

        let mut hovered_nodes = ui_stack
            .uinodes
            .iter()
            // reverse the iterator to traverse the tree from closest nodes to furthest
            .rev()
            .filter_map(|entity| {
                if let Ok(node) = node_query.get_mut(*entity) {
                    // Nodes that are not rendered should not be interactable
                    if let Some(view_visibility) = node.view_visibility {
                        if !view_visibility.get() {
                            return None;
                        }
                    }

                    let node_rect = node.node.logical_rect(node.global_transform);
                    let visible_rect = node
                        .calculated_clip
                        .map(|clip| node_rect.intersect(clip.clip))
                        .unwrap_or(node_rect);
                    if visible_rect.contains(location.position) {
                        Some(*entity)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<Entity>>()
            .into_iter();

        // As soon as a node with a `Block` focus policy is detected, the iteration will stop on it
        // because it "captures" the interaction.
        let mut iter = node_query.iter_many_mut(hovered_nodes.by_ref());
        let mut picks = Vec::new();
        let mut depth = 0.0;

        while let Some(node) = iter.fetch_next() {
            let mut push_hit =
                || picks.push((node.entity, HitData::new(*camera_entity, depth, None, None)));
            push_hit();
            if let Some(pickable) = node.pickable {
                // If an entity has a `Pickable` component, we will use that as the source of truth.
                if pickable.should_block_lower {
                    break;
                }
            } else {
                // If the Pickable component doesn't exist, default behavior is to block.
                break;
            }

            depth += 0.00001; // keep depth near 0 for precision
        }
        let order = camera.order as f32 + 0.5; // bevy ui can run on any camera, it's a special case
        output.send(PointerHits::new(*pointer, picks, order))
    }
}
