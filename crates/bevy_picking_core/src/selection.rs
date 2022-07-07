use bevy::prelude::*;

use crate::{
    input::PointerMultiselect,
    output::{Just, PointerInteractionEvent},
    PointerId,
};

/// Tracks the current selection state to be used with change tracking in the events system.
/// Entities with [Selection] will have selection state managed.
#[derive(Component, Debug, Default, Clone)]
pub struct PickSelection {
    pub is_selected: bool,
}

#[derive(Component, Debug, Copy, Clone)]
pub enum PointerSelectionEvent {
    JustSelected(Entity),
    JustDeselected(Entity),
}
impl PointerSelectionEvent {
    pub fn receive(
        mut events: EventReader<PointerSelectionEvent>,
        mut selectables: Query<&mut PickSelection>,
    ) {
        for event in events.iter() {
            match event {
                PointerSelectionEvent::JustSelected(entity) => {
                    if let Ok(mut s) = selectables.get_mut(*entity) {
                        s.is_selected = true
                    }
                }
                PointerSelectionEvent::JustDeselected(entity) => {
                    if let Ok(mut s) = selectables.get_mut(*entity) {
                        s.is_selected = false
                    }
                }
            }
        }
    }
}

/// Marker struct used to mark pickable entities for which you don't want to trigger a deselection
/// event when picked. This is useful for gizmos or other pickable UI entities.
#[derive(Component, Debug, Copy, Clone)]
pub struct NoDeselect;

pub fn send_selection_events(
    mut interactions: EventReader<PointerInteractionEvent>,
    pointers: Query<(&PointerId, &PointerMultiselect)>,
    no_deselect: Query<&NoDeselect>,
    selectables: Query<(Entity, &PickSelection)>,
    mut selection_events: EventWriter<PointerSelectionEvent>,
) {
    for interaction in interactions.iter() {
        let multiselect = pointers
            .iter()
            .find_map(|(id, ms)| id.eq(&interaction.id).then_some(ms.is_pressed))
            .unwrap_or(false);

        let entity_can_deselect = no_deselect.get(interaction.pick_entity).is_err();

        match interaction.event {
            Just::Down => {
                if !multiselect && entity_can_deselect {
                    for (entity, selection) in selectables.iter() {
                        if selection.is_selected {
                            selection_events.send(PointerSelectionEvent::JustDeselected(entity))
                        }
                    }
                }
            }
            Just::Clicked => {
                if let Ok((entity, selection)) = selectables.get(interaction.pick_entity) {
                    if multiselect {
                        match selection.is_selected {
                            true => {
                                selection_events.send(PointerSelectionEvent::JustDeselected(entity))
                            }
                            false => {
                                selection_events.send(PointerSelectionEvent::JustSelected(entity))
                            }
                        }
                    } else if !selection.is_selected {
                        selection_events.send(PointerSelectionEvent::JustSelected(entity))
                    }
                }
            }
            _ => (),
        }
    }
}
