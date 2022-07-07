use std::collections::BTreeMap;

use crate::{
    backend::{self, PickLayer},
    input,
    output::{Just, PickInteraction},
    PointerId, PointerInteractionEvent,
};
use bevy::{
    prelude::*,
    ui::FocusPolicy,
    utils::{FloatOrd, HashMap, HashSet},
};

pub fn update_focus(
    focus: Query<&FocusPolicy>,
    pointers: Query<&PointerId>,
    pick_layers: Query<&PickLayer>,
    mut click_events: EventReader<input::PointerClickEvent>,
    mut over_events: EventReader<backend::PointerOverEvent>,
    // Local
    mut pointer_map: Local<HashMap<PointerId, BTreeMap<PickLayer, BTreeMap<FloatOrd, Entity>>>>,
    mut hover_set: Local<HashSet<Entity>>,
    // Outputs
    mut interaction_query: Query<(Entity, &mut PickInteraction)>,
    mut interactions: EventWriter<PointerInteractionEvent>,
) {
    // Clear local maps, reusing memory.
    for (_pointer, layer_map) in pointer_map.iter_mut() {
        for (_layer, depth_map) in layer_map.iter_mut() {
            depth_map.clear()
        }
    }
    // Build an ordered map of entities that are under each pointer
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
    // Build an unsorted set of hovered entities, accounting for depth, layer, and focus policy.
    for &id in pointers.iter() {
        if !hover_set.is_empty() {
            hover_set.clear();
        }
        if let Some(under_pointer) = pointer_map.get_mut(&id) {
            for (_layer, depth_map) in under_pointer.iter() {
                for (_depth, entity) in depth_map.iter() {
                    // Note that while this loop visits each entity in layer/depth order for
                    // purposes of focus policy, the set it produces is not in any order.
                    hover_set.insert(*entity);
                    if let Ok(_policy @ FocusPolicy::Block) = focus.get(*entity) {
                        break;
                    }
                }
            }
        }

        for (entity, mut interaction) in interaction_query.iter_mut() {
            let interaction = match interaction.map.get_mut(&id) {
                Some(i) => i,
                None => interaction.map.try_insert(id, Interaction::None).unwrap(),
            };
            *interaction = if hover_set.contains(&entity) {
                // The entity **is** being hovered by the pointer
                if matches!(interaction, Interaction::None) {
                    interactions.send(PointerInteractionEvent::new(entity, id, Just::Entered));
                }
                // Process interaction events, using the final event for the current state.
                click_events
                    .iter()
                    .filter_map(|click| {
                        if click.is_just_down(id) {
                            interactions.send(PointerInteractionEvent::new(entity, id, Just::Down));
                            Some(Interaction::Clicked)
                        } else if click.is_just_up(id) {
                            interactions.send(PointerInteractionEvent::new(entity, id, Just::Up));
                            Some(Interaction::Hovered)
                        } else {
                            None // This event is not for the current pointer, we need to filter it out!
                        }
                    })
                    .last()
                    .unwrap_or(Interaction::Hovered)
            } else {
                // The entity is **not** being hovered by the pointer
                if matches!(interaction, Interaction::Hovered | Interaction::Clicked) {
                    click_events.iter().for_each(|click| {
                        if click.is_just_up(id) {
                            // The pointer is *just* no longer over the entity, **and** the pointer
                            // was *just* released. This ensures touch releases are captured.
                            interactions.send(PointerInteractionEvent::new(entity, id, Just::Up));
                        }
                    });
                    interactions.send(PointerInteractionEvent::new(entity, id, Just::Exited));
                }
                Interaction::None
            };
        }
    }
}

/// Used to locally track the click state of an entity.
pub enum ClickState {
    None,
    DownReceived,
}

/// Sends click events when an entity receives a mouse down event followed by a mouse up event,
/// without receiving an exit event in between.
pub fn send_click_events(
    mut events: ParamSet<(
        EventReader<PointerInteractionEvent>,
        EventWriter<PointerInteractionEvent>,
    )>,
    mut click_states: Local<HashMap<Entity, ClickState>>,
) {
    let mut events_to_send = Vec::new();
    for interaction in events.p0().iter() {
        match interaction.event {
            Just::Down => {
                click_states.insert(interaction.pick_entity, ClickState::DownReceived);
            }
            Just::Up => {
                if matches!(
                    click_states.get(&interaction.pick_entity),
                    Some(ClickState::DownReceived)
                ) {
                    events_to_send.push(PointerInteractionEvent {
                        pick_entity: interaction.pick_entity,
                        id: interaction.id,
                        event: Just::Clicked,
                    });
                    click_states.insert(interaction.pick_entity, ClickState::None);
                }
            }
            Just::Exited => {
                click_states.insert(interaction.pick_entity, ClickState::None);
            }
            _ => (),
        }
    }
    events.p1().send_batch(events_to_send.into_iter());
}
