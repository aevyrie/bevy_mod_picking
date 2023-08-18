//! A picking backend for [`bevy_ui`](bevy::ui).
//!
//! # Usage
//!
//! This backend does not require markers on cameras or entities to function. It will look for any
//! pointers using the same render target as the UI camera, and run hit tests on the UI node tree.
//!
//! ## Important Note
//!
//! This backend completely ignores [`FocusPolicy`](bevy::ui::FocusPolicy). The design of bevy ui's
//! focus systems and the picking plugin are not compatible. Instead, use the [`Pickable`] component
//! to customize how an entity responds to picking focus.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::camera::NormalizedRenderTarget,
    ui::{RelativeCursorPosition, UiStack},
    window::PrimaryWindow,
};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_ui`](crate) crate.
pub mod prelude {
    pub use crate::BevyUiBackend;
}

/// For some reason bevy_ui seems to ignore the camera order of the UI camera, so we need to pick
/// something arbitrary to send in [`PointerHits`] so that bevy_ui hits can be grouped together.
/// From what I can tell, bevy_ui always renders on top, so we will just set this to max. 🥲
pub const BEVY_UI_CAMERA_ORDER: isize = isize::MAX;

/// Adds picking support for [`bevy_ui`](bevy::ui)
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
    computed_visibility: Option<&'static ComputedVisibility>,
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
    mut node_query: Query<NodeQuery>,
    mut output: EventWriter<PointerHits>,
) {
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
            .map(|loc| (pointer, loc))
    }) {
        let window_entity = primary_window.single();

        // Find the camera with the same target as this pointer
        let Some((camera_entity, ui_config)) = cameras
            .iter()
            .find(|(_entity, camera, _)| {
                camera.target.normalize(Some(window_entity)).unwrap() == location.target
            })
            .map(|(entity, _camera, ui_config)| (entity, ui_config)) else {
                continue;
            };

        // If this ui camera is disabled, skip to the next pointer.
        if matches!(ui_config, Some(&UiCameraConfig { show_ui: false, .. })) {
            continue;
        }

        let mut hovered_nodes = ui_stack
            .uinodes
            .iter()
            // reverse the iterator to traverse the tree from closest nodes to furthest
            .rev()
            .filter_map(|entity| {
                if let Ok(node) = node_query.get_mut(*entity) {
                    // Nodes that are not rendered should not be interactable
                    if let Some(computed_visibility) = node.computed_visibility {
                        if !computed_visibility.is_visible() {
                            return None;
                        }
                    }

                    let position = node.global_transform.translation();
                    let ui_position = position.truncate();
                    let extents = node.node.size() / 2.0;
                    let mut min = ui_position - extents;
                    if let Some(clip) = node.calculated_clip {
                        min = Vec2::max(min, clip.clip.min);
                    }

                    // The mouse position relative to the node
                    // (0., 0.) is the top-left corner, (1., 1.) is the bottom-right corner
                    let relative_cursor_position = Vec2::new(
                        (location.position.x - min.x) / node.node.size().x,
                        (location.position.y - min.y) / node.node.size().y,
                    );

                    if (0.0..1.).contains(&relative_cursor_position.x)
                        && (0.0..1.).contains(&relative_cursor_position.y)
                    {
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
            let mut push_hit = || {
                picks.push((
                    node.entity,
                    HitData {
                        camera: camera_entity,
                        depth,
                        position: None,
                        normal: None,
                    },
                ))
            };
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

        output.send(PointerHits {
            pointer: *pointer,
            picks,
            order: BEVY_UI_CAMERA_ORDER,
        })
    }
}
