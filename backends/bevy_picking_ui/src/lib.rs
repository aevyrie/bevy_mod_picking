//! A raycasting backend for [`bevy_ui`](bevy::ui).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::camera::NormalizedRenderTarget,
    ui::{FocusPolicy, RelativeCursorPosition, UiStack},
    window::PrimaryWindow,
};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_ui`](crate) crate.
pub mod prelude {
    pub use crate::BevyUiBackend;
}

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
    interaction: Option<&'static mut Interaction>,
    relative_cursor_position: Option<&'static mut RelativeCursorPosition>,
    focus_policy: Option<&'static FocusPolicy>,
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
                    if primary_window.get(window.entity()).is_ok() {
                        return true;
                    }
                }
                false
            })
            .map(|loc| (pointer, loc))
    }) {
        let window_entity = primary_window.single();
        let Some((camera, ui_config)) = cameras
            .iter()
            .find(|(_entity, camera, _)| {
                camera.target.normalize(Some(window_entity)).unwrap() == location.target
            })
            .map(|(entity, _camera, ui_config)| (entity, ui_config)) else {
                continue;
            };

        if matches!(ui_config, Some(&UiCameraConfig { show_ui: false, .. })) {
            return;
        }

        let cursor_position = location.position;

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
                    // (0., 0.) is the bottom-left corner, (1., 1.) is the top-right corner
                    let relative_cursor_position = Vec2::new(
                        (cursor_position.x - min.x) / node.node.size().x,
                        (cursor_position.y - min.y) / node.node.size().y,
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
            picks.push((
                node.entity,
                HitData {
                    camera,
                    depth,
                    position: None,
                    normal: None,
                },
            ));
            match node.focus_policy.unwrap_or(&FocusPolicy::Block) {
                FocusPolicy::Block => {
                    break;
                }
                FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
            }
            depth += 0.00001; // keep depth near 0 for precision
        }

        output.send(PointerHits {
            pointer: *pointer,
            picks,
            order: 10,
        })
    }
}
