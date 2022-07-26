//! Determines which entities are being hovered by pointers, taking into account [`FocusPolicy`],
//! [`PickLayer`], and entity depth.

use std::collections::BTreeMap;

use crate::{backend, output::PointerInteraction, pointer::PointerId};
use bevy::{
    prelude::*,
    ui::FocusPolicy,
    utils::{FloatOrd, HashMap, HashSet},
};

/// A map of entities sorted by depth.
type DepthMap = BTreeMap<FloatOrd, Entity>;

/// Maps [`PickLayer`]s to the map of entities within that pick layer, sorted by depth.
type LayerMap = BTreeMap<PickLayer, DepthMap>;

/// Assigns an entity to a picking layer. When computing picking focus, entities
/// are sorted in order from the highest to lowest layer, and by depth within each layer.
#[derive(Debug, Clone, Copy, Component, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickLayer {
    /// Topmost picking layer
    Top = 0,
    /// Layer immediately above the UI layer
    AboveUi = 1,
    /// Contains all UI entities
    UI = 2,
    /// Layer immediately below the UI
    BelowUi = 3,
    /// Layer immediately above the world
    AboveWorld = 4,
    /// The default `PickLayer`.
    #[default]
    World = 5,
    /// Layer immediately below the world
    BelowWorld = 6,
    /// Bottommost picking layer
    Bottom = 7,
}

/// Maps Pointers to a [`LayerMap`]. Note this is much more complex than the [`HoverMap`] because
/// this data structure is used to sort entities by layer then depth for every pointer.
type OverMap = HashMap<PointerId, LayerMap>;

/// Maps pointers to the entities they are hovering over. "Hovering" refers to the actual hover
/// interaction state, in this case, not just whether the pointer happens to be over the entity.
#[derive(Debug, Deref, DerefMut, Default)]
pub struct HoverMap(pub HashMap<PointerId, HashSet<Entity>>);

/// Coalesces all data from inputs and backends to generate a map of the currently hovered entities.
pub fn update_focus(
    // Inputs
    focus: Query<&FocusPolicy>,
    pick_layers: Query<&PickLayer>,
    pointers: Query<(&PointerId, &PointerInteraction)>, // <- what happened last frame
    mut under_pointer: EventReader<backend::EntitiesUnderPointer>,
    // Local
    mut pointer_map: Local<OverMap>,
    // Output
    mut hover_map: ResMut<HoverMap>,
) {
    reset_local_maps(&mut hover_map, &mut pointer_map);
    build_pointer_map(pick_layers, &mut under_pointer, &mut pointer_map);
    build_hover_map(&pointers, focus, pointer_map, &mut hover_map);
}

/// Clear non-empty local maps, reusing allocated memory.
fn reset_local_maps(hover_map: &mut ResMut<HoverMap>, pointer_map: &mut Local<OverMap>) {
    for entity_set in hover_map.values_mut() {
        if !entity_set.is_empty() {
            entity_set.clear()
        }
    }
    for layer_map in pointer_map.values_mut() {
        for depth_map in layer_map.values_mut() {
            if !depth_map.is_empty() {
                depth_map.clear()
            }
        }
    }
}

/// Build an ordered map of entities that are under each pointer
fn build_pointer_map(
    pick_layers: Query<&PickLayer>,
    over_events: &mut EventReader<backend::EntitiesUnderPointer>,
    pointer_map: &mut Local<OverMap>,
) {
    for event in over_events.iter() {
        let layer_map = match pointer_map.get_mut(&event.id) {
            Some(map) => map,
            None => pointer_map.try_insert(event.id, BTreeMap::new()).unwrap(),
        };
        for over in event.over_list.iter() {
            let layer = pick_layers
                .get(over.entity)
                .map(|layer| *layer)
                .unwrap_or_else(|_error| {
                    error!(
                        "Pickable entity {:?} doesn't have a `PickLayer` component",
                        over.entity
                    );
                    PickLayer::default()
                });

            layer_map.entry(layer).or_insert_with(BTreeMap::new);

            let depth_map = layer_map.get_mut(&layer).unwrap();
            depth_map.insert(FloatOrd(over.depth), over.entity);
        }
    }
}

// Build an unsorted set of hovered entities, accounting for depth, layer, and focus policy. Note
// that unlike the pointer map, this uses the focus policy to determine if lower entities receive
// hover focus. Often, only a single entity per pointer will be hovered.
fn build_hover_map(
    pointers: &Query<(&PointerId, &PointerInteraction)>,
    focus: Query<&FocusPolicy>,
    pointer_map: Local<OverMap>,
    // Output
    hover_map: &mut ResMut<HoverMap>,
) {
    for (id, _) in pointers.iter() {
        let pointer_entity_set = match hover_map.contains_key(id) {
            true => hover_map.get_mut(id).unwrap(),
            false => hover_map.try_insert(*id, HashSet::new()).unwrap(),
        };
        if let Some(layer_map) = pointer_map.get(id) {
            for depth_map in layer_map.values() {
                for entity in depth_map.values() {
                    pointer_entity_set.insert(*entity);
                    if let Ok(_policy @ FocusPolicy::Block) = focus.get(*entity) {
                        break;
                    }
                }
            }
        }
    }
}
