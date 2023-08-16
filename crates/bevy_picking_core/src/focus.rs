//! Determines which entities are being hovered by which pointers.

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use crate::{
    backend::{self, HitData},
    events::{Down, IsPointerEvent, Out, Over, Pointer, PointerCancel, Up},
    pointer::PointerId,
};
use bevy::{
    prelude::*,
    ui::FocusPolicy,
    utils::{FloatOrd, HashMap},
};

/// A map of entities sorted by depth.
type DepthMap = BTreeMap<FloatOrd, (Entity, HitData)>;

/// Events returned from backends can be grouped with an order field. This allows picking to work
/// with multiple layers of rendered output to the same render target.
type PickLayer = isize;

/// Maps [`RenderLayers`] to the map of entities within that pick layer, sorted by depth.
type LayerMap = BTreeMap<PickLayer, DepthMap>;

/// Maps Pointers to a [`LayerMap`]. Note this is much more complex than the [`HoverMap`] because
/// this data structure is used to sort entities by layer then depth for every pointer.
type OverMap = HashMap<PointerId, LayerMap>;

/// Maps pointers to the entities they are hovering over. "Hovering" refers to the actual hover
/// state, in this case, not just whether the pointer happens to be over the entity. More
/// specifically, a pointer is "over" an entity if it is within the bounds of that entity, whereas a
/// pointer is "hovering" an entity only if the mouse is "over" the entity AND it is the topmost
/// entity.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct HoverMap(pub HashMap<PointerId, HashMap<Entity, HitData>>);

/// The previous state of the hover map, used to track changes to hover state.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct PreviousHoverMap(pub HashMap<PointerId, HashMap<Entity, HitData>>);

/// Coalesces all data from inputs and backends to generate a map of the currently hovered entities.
/// This is the final focusing step to determine which entity the pointer is hovering over.
pub fn update_focus(
    // Inputs
    focus: Query<&FocusPolicy>,
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
        for &(entity, pick_data) in entities_under_pointer.picks.iter() {
            let layer = entities_under_pointer.order;
            let depth_map = layer_map.entry(layer).or_insert_with(BTreeMap::new);
            depth_map.insert(FloatOrd(pick_data.depth), (entity, pick_data));
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
        let pointer_entity_set = hover_map.entry(*pointer_id).or_insert_with(HashMap::new);
        if let Some(layer_map) = over_map.get(pointer_id) {
            // Note we reverse here to start from the highest layer first.
            for &(entity, pick_data) in layer_map
                .values()
                .rev()
                .flat_map(|depth_map| depth_map.values())
            {
                pointer_entity_set.insert(entity, pick_data);
                if let Ok(FocusPolicy::Block) = focus.get(entity) {
                    break;
                }
                if focus.get(entity).is_err() {
                    break;
                }
            }
        }
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
        update_interactions(event, Interaction::Pressed, &mut pointers, &mut interact);
    }
    for event in pointer_up.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut interact);
    }
    for event in pointer_out.iter() {
        update_interactions(event, Interaction::None, &mut pointers, &mut interact);
    }
}

fn update_interactions<E: IsPointerEvent>(
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
