//! A raycasting backend for [`bevy_ui`](bevy::ui).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::camera::NormalizedRenderTarget,
    ui::{FocusPolicy, RelativeCursorPosition, UiStack},
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_picking_core::{
    backend::prelude::*,
    events::{Down, Out, Over, Pointer, Up},
};

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
        app.add_system(ui_picking.in_set(PickSet::Backend))
            .add_system(interactions_from_events.in_set(PickSet::Focus));
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
    primary_window: Query<(Entity, &Window), With<PrimaryWindow>>,
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
        let (window_entity, window) = primary_window.single();
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

        let mut cursor_position = location.position;
        cursor_position.y = window.resolution.height() - cursor_position.y;

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
            let mut push_hit = || {
                picks.push((
                    node.entity,
                    HitData {
                        camera,
                        depth,
                        position: None,
                        normal: None,
                    },
                ))
            };
            push_hit();
            if let Some(pickable) = node.pickable {
                // If an entity has a `Pickable` component, we will use that as the source of truth.
                if pickable.is_blocker {
                    break;
                }
            } else if let Some(focus_policy) = node.focus_policy {
                // Fall back to using bevy's `FocusPolicy` if there is no `Pickable`.
                match focus_policy {
                    FocusPolicy::Block => break,
                    FocusPolicy::Pass => (), // allow the next node to be hovered/clicked
                }
            } else {
                // If neither component exists, default behavior is to block.
                break;
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

/// Holds a map of entities this pointer is currently interacting with.
#[derive(Debug, Default, Clone, Component)]
pub struct PointerInteraction {
    map: HashMap<Entity, Interaction>,
}
impl Deref for PointerInteraction {
    type Target = HashMap<Entity, Interaction>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl DerefMut for PointerInteraction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// Uses pointer events to update [`PointerInteraction`] and [`Interaction`] components.
pub fn interactions_from_events(
    // Input
    mut pointer_over: EventReader<Pointer<Over>>,
    mut pointer_out: EventReader<Pointer<Out>>,
    mut pointer_up: EventReader<Pointer<Up>>,
    mut pointer_down: EventReader<Pointer<Down>>,
    // Outputs
    mut pointers: Query<(&PointerId, &mut PointerInteraction)>,
    mut interact: Query<&mut Interaction>,
) {
    for event in pointer_over.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut interact);
    }
    for event in pointer_down.iter() {
        update_interactions(event, Interaction::Clicked, &mut pointers, &mut interact);
    }
    for event in pointer_up.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut interact);
    }
    for event in pointer_out.iter() {
        update_interactions(event, Interaction::None, &mut pointers, &mut interact);
    }
}

fn update_interactions<E: Debug + Clone + Reflect>(
    event: &Pointer<E>,
    new_interaction: Interaction,
    pointer_interactions: &mut Query<(&PointerId, &mut PointerInteraction)>,
    entity_interactions: &mut Query<&mut Interaction>,
) {
    if let Some(mut interaction_map) = pointer_interactions
        .iter_mut()
        .find_map(|(id, interaction)| (*id == event.pointer_id).then_some(interaction))
    {
        interaction_map.insert(event.target, new_interaction);
        if let Ok(mut interaction) = entity_interactions.get_mut(event.target) {
            *interaction = new_interaction;
        }
        interaction_map
            .retain(|_, i| i != &Interaction::None || new_interaction != Interaction::None);
    };
}
