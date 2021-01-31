use crate::{Hover, PickableMesh, RayCastPluginState, Selection};
use bevy::prelude::*;

/// An event that triggers when the hover state of a [Selection] enabled [PickableMesh] changes.
#[derive(Debug)]
pub enum SelectionEvent {
    JustSelected(Entity),
    JustDeselected(Entity),
}

/// An event that triggers when the hover state of a [Hover] enabled [PickableMesh] changes.
#[derive(Debug)]
pub enum HoverEvent {
    JustEntered(Entity),
    JustLeft(Entity),
}

/// Looks for changes in selection or hover state, and sends the appropriate events
pub fn mesh_events_system(
    state: Res<RayCastPluginState>,
    mut selection_events: ResMut<Events<SelectionEvent>>,
    mut hover_events: ResMut<Events<HoverEvent>>,
    hover_query: Query<(Entity, &Hover), (Changed<Hover>, With<PickableMesh>)>,
    selection_query: Query<(Entity, &Selection), (Changed<Selection>, With<PickableMesh>)>,
) {
    if !state.enabled {
        return;
    }
    for (entity, hover) in hover_query.iter() {
        if hover.hovered() {
            hover_events.send(HoverEvent::JustEntered(entity));
        } else {
            hover_events.send(HoverEvent::JustLeft(entity))
        }
    }
    for (entity, selection) in selection_query.iter() {
        if selection.selected() {
            selection_events.send(SelectionEvent::JustSelected(entity));
        } else {
            selection_events.send(SelectionEvent::JustDeselected(entity))
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system(
    state: Res<RayCastPluginState>,
    mut hover_reader: EventReader<HoverEvent>,
    mut selection_reader: EventReader<SelectionEvent>,
) {
    if !state.enabled {
        return;
    }
    for event in hover_reader.iter() {
        println!("{:?}", event);
    }
    for event in selection_reader.iter() {
        println!("{:?}", event);
    }
}
