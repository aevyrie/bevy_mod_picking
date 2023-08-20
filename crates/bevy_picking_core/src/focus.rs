//! Determines which entities are being hovered by which pointers.

use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    backend::{self, HitData},
    events::PointerCancel,
    pointer::{PointerId, PointerInteraction, PointerPress},
    Pickable,
};

use bevy::{
    prelude::*,
    utils::{FloatOrd, HashMap},
};

/// A map of entities sorted by depth.
type DepthMap = BTreeMap<FloatOrd, (Entity, HitData)>;

/// Events returned from backends can be grouped with an order field. This allows picking to work
/// with multiple layers of rendered output to the same render target.
type PickLayer = FloatOrd;

/// Maps [`RenderLayers`] to the map of entities within that pick layer, sorted by depth.
type LayerMap = BTreeMap<PickLayer, DepthMap>;

/// Maps Pointers to a [`LayerMap`]. Note this is much more complex than the [`HoverMap`] because
/// this data structure is used to sort entities by layer then depth for every pointer.
type OverMap = HashMap<PointerId, LayerMap>;

/// Maps pointers to the entities they are hovering over.
///
/// "Hovering" refers to the *hover* state, which is not the same as whether or not the pointer
/// happens to be over the entity. More specifically, a pointer is "over" an entity if it is within
/// the bounds of that entity, whereas a pointer is "hovering" an entity only if the mouse is "over"
/// the entity *and* no entities between it and the pointer block interactions.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct HoverMap(pub HashMap<PointerId, HashMap<Entity, HitData>>);

/// The previous state of the hover map, used to track changes to hover state.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct PreviousHoverMap(pub HashMap<PointerId, HashMap<Entity, HitData>>);

/// Coalesces all data from inputs and backends to generate a map of the currently hovered entities.
/// This is the final focusing step to determine which entity the pointer is hovering over.
pub fn update_focus(
    // Inputs
    pickable: Query<&Pickable>,
    pointers: Query<&PointerId>,
    mut under_pointer: EventReader<backend::PointerHits>,
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
    build_over_map(&mut under_pointer, &mut over_map, &mut cancellations);
    build_hover_map(&pointers, pickable, &over_map, &mut hover_map);
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
        layer_map.clear()
    }

    // Clear pointers from the maps if they have been removed.
    let active_pointers: Vec<PointerId> = pointers.iter().copied().collect();
    hover_map.retain(|pointer, _| active_pointers.contains(pointer));
    over_map.retain(|pointer, _| active_pointers.contains(pointer));
}

/// Build an ordered map of entities that are under each pointer
fn build_over_map(
    backend_events: &mut EventReader<backend::PointerHits>,
    pointer_over_map: &mut Local<OverMap>,
    pointer_cancel: &mut EventReader<PointerCancel>,
) {
    let cancelled_pointers: Vec<PointerId> = pointer_cancel.iter().map(|p| p.pointer_id).collect();

    for entities_under_pointer in backend_events
        .iter()
        .filter(|e| !cancelled_pointers.contains(&e.pointer))
    {
        let pointer = entities_under_pointer.pointer;
        let layer_map = pointer_over_map
            .entry(pointer)
            .or_insert_with(BTreeMap::new);
        for (entity, pick_data) in entities_under_pointer.picks.iter() {
            let layer = entities_under_pointer.order;
            let depth_map = layer_map
                .entry(FloatOrd(layer))
                .or_insert_with(BTreeMap::new);
            depth_map.insert(FloatOrd(pick_data.depth), (*entity, pick_data.clone()));
        }
    }
}

/// Build an unsorted set of hovered entities, accounting for depth, layer, and [`Pickable`]. Note
/// that unlike the pointer map, this uses [`Pickable`] to determine if lower entities receive hover
/// focus. Often, only a single entity per pointer will be hovered.
fn build_hover_map(
    pointers: &Query<&PointerId>,
    pickable: Query<&Pickable>,
    over_map: &Local<OverMap>,
    // Output
    hover_map: &mut HoverMap,
) {
    for pointer_id in pointers.iter() {
        let pointer_entity_set = hover_map.entry(*pointer_id).or_insert_with(HashMap::new);
        if let Some(layer_map) = over_map.get(pointer_id) {
            // Note we reverse here to start from the highest layer first.
            for (entity, pick_data) in layer_map
                .values()
                .rev()
                .flat_map(|depth_map| depth_map.values())
            {
                if let Ok(pickable) = pickable.get(*entity) {
                    if pickable.should_emit_events {
                        pointer_entity_set.insert(*entity, pick_data.clone());
                    }
                    if pickable.should_block_lower {
                        break;
                    }
                } else {
                    pointer_entity_set.insert(*entity, pick_data.clone()); // Emit events by default
                    break; // Entities block by default so we break out of the loop
                }
            }
        }
    }
}

/// A component that aggregates picking interaction state of this entity across all pointers.
///
/// Unlike bevy's `Interaction` component, this is an aggregate of the state of all pointers
/// interacting with this entity. Aggregation is done by taking the interaction with the highest
/// precedence.
///
/// For example, if we have an entity that is being hovered by one pointer, and pressed by another,
/// the entity will be considered pressed. If that entity is instead being hovered by both pointers,
/// it will be considered hovered.
#[derive(Component, Copy, Clone, Default, Eq, PartialEq, Debug, Reflect)]
pub enum PickingInteraction {
    /// The entity is being pressed down by a pointer.
    Pressed = 2,
    /// The entity is being hovered by a pointer.
    Hovered = 1,
    /// No pointers are interacting with this entity.
    #[default]
    None = 0,
}

/// Uses pointer events to update [`PointerInteraction`] and [`PickingInteraction`] components.
pub fn update_interactions(
    // Input
    hover_map: Res<HoverMap>,
    previous_hover_map: Res<PreviousHoverMap>,
    // Outputs
    mut commands: Commands,
    mut pointers: Query<(&PointerId, &PointerPress, &mut PointerInteraction)>,
    mut interact: Query<&mut PickingInteraction>,
) {
    // Clear all previous hover data from pointers and entities
    for (pointer, _, mut pointer_interaction) in &mut pointers {
        pointer_interaction.sorted_entities.clear();
        if let Some(previously_hovered_entities) = previous_hover_map.get(pointer) {
            for entity in previously_hovered_entities.keys() {
                if let Ok(mut interaction) = interact.get_mut(*entity) {
                    *interaction = PickingInteraction::None;
                }
            }
        }
    }

    // Create a map to hold the aggregated interaction for each entity. This is needed because we
    // need to be able to insert the interaction component on entities if they do not exist. To do
    // so we need to know the final aggregated interaction state to avoid the scenario where we set
    // an entity to `Pressed`, then overwrite that with a lower precedent like `Hovered`.
    let mut new_interaction_state = HashMap::<Entity, PickingInteraction>::new();
    for (pointer, pointer_press, mut pointer_interaction) in &mut pointers {
        if let Some(pointers_hovered_entities) = hover_map.get(pointer) {
            // Insert a sorted list of hit entities into the pointer's interaction component.
            let mut sorted_entities: Vec<_> = pointers_hovered_entities.clone().drain().collect();
            sorted_entities.sort_by_key(|(_entity, hit)| FloatOrd(hit.depth));
            pointer_interaction.sorted_entities = sorted_entities;

            for hovered_entity in pointers_hovered_entities.iter().map(|(entity, _)| entity) {
                merge_interaction_states(pointer_press, hovered_entity, &mut new_interaction_state);
            }
        }
    }

    // Take the aggregated entity states and update or insert the component if missing.
    for (hovered_entity, new_interaction) in new_interaction_state.drain() {
        if let Ok(mut interaction) = interact.get_mut(hovered_entity) {
            *interaction = new_interaction;
        } else if let Some(mut entity_commands) = commands.get_entity(hovered_entity) {
            entity_commands.insert(new_interaction);
        }
    }
}

/// Merge the interaction state of this entity into the aggregated map.
fn merge_interaction_states(
    pointer_press: &PointerPress,
    hovered_entity: &Entity,
    new_interaction_state: &mut HashMap<Entity, PickingInteraction>,
) {
    let new_interaction = match pointer_press.is_any_pressed() {
        true => PickingInteraction::Pressed,
        false => PickingInteraction::Hovered,
    };

    if let Some(old_interaction) = new_interaction_state.get_mut(hovered_entity) {
        // Only update if the new value has a higher precedence than the old value.
        if *old_interaction != new_interaction
            && matches!(
                (*old_interaction, new_interaction),
                (PickingInteraction::Hovered, PickingInteraction::Pressed)
                    | (PickingInteraction::None, PickingInteraction::Pressed)
                    | (PickingInteraction::None, PickingInteraction::Hovered)
            )
        {
            *old_interaction = new_interaction;
        }
    } else {
        new_interaction_state.insert(*hovered_entity, new_interaction);
    }
}
