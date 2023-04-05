//! Determines which entities are being hovered by pointers, taking into account [`FocusPolicy`],
//! [`RenderLayers`], and entity depth.

use std::collections::BTreeMap;

use crate::{backend, events::PointerCancel, pointer::PointerId};
use bevy::{
    prelude::*,
    render::view::{Layer, RenderLayers},
    ui::FocusPolicy,
    utils::{FloatOrd, HashMap, HashSet},
};

/// A map of entities sorted by depth.
type DepthMap = BTreeMap<FloatOrd, Entity>;

/// Maps [`RenderLayers`] to the map of entities within that pick layer, sorted by depth.
type LayerMap = BTreeMap<Layer, DepthMap>;

/// Maps Pointers to a [`LayerMap`]. Note this is much more complex than the [`HoverMap`] because
/// this data structure is used to sort entities by layer then depth for every pointer.
type OverMap = HashMap<PointerId, LayerMap>;

/// Maps pointers to the entities they are hovering over. "Hovering" refers to the actual hover
/// state, in this case, not just whether the pointer happens to be over the entity. More
/// specifically, a pointer is "over" an entity if it is within the bounds of that entity, whereas a
/// pointer is "hovering" an entity only if the mouse is "over" the entity AND it is the topmost
/// entity(s) according to `FocusPolicy` and `RenderLayer`.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct HoverMap(pub HashMap<PointerId, HashSet<Entity>>);

/// The previous state of the hover map, used to track changes to hover state.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct PreviousHoverMap(pub HashMap<PointerId, HashSet<Entity>>);

/// Coalesces all data from inputs and backends to generate a map of the currently hovered entities.
/// This is the final focusing step to determine which entity the pointer is hovering over.
pub fn update_focus(
    // Inputs
    focus: Query<&FocusPolicy>,
    render_layers: Query<&RenderLayers>,
    pointers: Query<&PointerId>,
    mut under_pointer: EventReader<backend::EntitiesUnderPointer>,
    mut cancellations: EventReader<PointerCancel>,
    // Local
    mut over_map: Local<OverMap>,
    // Output
    mut hover_map: ResMut<HoverMap>,
    mut previous_hover_map: ResMut<PreviousHoverMap>,
) {
    reset_maps(
        &mut hover_map,
        &mut previous_hover_map,
        &mut over_map,
        &pointers,
    );
    build_over_map(
        render_layers,
        &mut under_pointer,
        &mut over_map,
        &mut cancellations,
    );
    build_hover_map(&pointers, focus, &over_map, &mut hover_map);
}

/// Clear non-empty local maps, reusing allocated memory.
fn reset_maps(
    hover_map: &mut HoverMap,
    previous_hover_map: &mut PreviousHoverMap,
    over_map: &mut OverMap,
    pointers: &Query<&PointerId>,
) {
    // Swap the previous and current hover maps. This results in the previous values being stored in
    // `PreviousHoverMap`. Swapping is okay because we clear the `HoverMap` which now holds stale
    // data. This process is done without any allocations.
    core::mem::swap(&mut previous_hover_map.0, &mut hover_map.0);

    for entity_set in hover_map.values_mut() {
        entity_set.clear()
    }
    for layer_map in over_map.values_mut() {
        for depth_map in layer_map.values_mut() {
            depth_map.clear()
        }
    }

    // Clear pointers from the maps if they have been removed.
    let active_pointers: Vec<PointerId> = pointers.iter().copied().collect();
    hover_map.retain(|pointer, _| active_pointers.contains(pointer));
    over_map.retain(|pointer, _| active_pointers.contains(pointer));
}

/// Build an ordered map of entities that are under each pointer
fn build_over_map(
    render_layers: Query<&RenderLayers>,
    backend_events: &mut EventReader<backend::EntitiesUnderPointer>,
    pointer_over_map: &mut Local<OverMap>,
    pointer_cancel: &mut EventReader<PointerCancel>,
) {
    let cancelled_pointers: Vec<PointerId> = pointer_cancel.iter().map(|p| p.pointer_id).collect();

    for entities_under_pointer in backend_events
        .iter()
        .filter(|e| !cancelled_pointers.contains(&e.pointer))
    {
        let layer_map = pointer_over_map
            .entry(entities_under_pointer.pointer)
            .or_insert_with(BTreeMap::new);
        for over in entities_under_pointer.over_list.iter() {
            let layer: Layer = render_layers
                .get(over.entity)
                .ok()
                .and_then(|layer| layer.iter().max())
                .unwrap_or(0);
            let depth_map = layer_map.entry(layer).or_insert_with(BTreeMap::new);
            depth_map.insert(FloatOrd(over.depth), over.entity);
        }
    }
}

/// Build an unsorted set of hovered entities, accounting for depth, layer, and focus policy. Note
/// that unlike the pointer map, this uses the focus policy to determine if lower entities receive
/// hover focus. Often, only a single entity per pointer will be hovered.
fn build_hover_map(
    pointers: &Query<&PointerId>,
    focus: Query<&FocusPolicy>,
    over_map: &Local<OverMap>,
    // Output
    hover_map: &mut HoverMap,
) {
    for pointer_id in pointers.iter() {
        let pointer_entity_set = hover_map.entry(*pointer_id).or_insert_with(HashSet::new);
        if let Some(layer_map) = over_map.get(pointer_id) {
            // Note we reverse here to start from the highest layer first
            for depth_map in layer_map.values().rev() {
                for entity in depth_map.values() {
                    pointer_entity_set.insert(*entity);
                    if let Ok(FocusPolicy::Block) = focus.get(*entity) {
                        break;
                    }
                    if focus.get(*entity).is_err() {
                        break;
                    }
                }
            }
        }
    }
}
