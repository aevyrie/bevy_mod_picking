use crate::{hit::CursorHit, input::CursorInput, Hover, Selection};
use bevy::prelude::*;

/// An event that triggers when the selection state of a [Selection] enabled [PickableTarget] changes.
#[derive(Debug)]
pub enum SelectionEvent {
    JustSelected(Entity),
    JustDeselected(Entity),
}

/// An event that triggers when the hover state of a [Hover] enabled [PickableTarget] changes.
#[derive(Debug)]
pub enum HoverEvent {
    JustEntered(Entity),
    JustLeft(Entity),
}

/// An event that wraps selection and hover events
#[derive(Debug)]
pub enum PickingEvent {
    Selection(SelectionEvent),
    Hover(HoverEvent),
    Clicked(Entity),
}

/// Looks for changes in selection or hover state, and sends the appropriate events
#[allow(clippy::type_complexity)]
pub fn write_events(
    hover_query: Query<(Entity, &Hover, ChangeTrackers<Hover>), Changed<Hover>>,
    selection_query: Query<(Entity, &Selection, ChangeTrackers<Selection>), Changed<Selection>>,
    cursors: Query<(&CursorInput, &CursorHit)>,
    mut picking_events: EventWriter<PickingEvent>,
) {
    for (entity, hover, hover_change) in hover_query.iter() {
        if hover_change.is_added() {
            continue; // Avoid a false change detection when a component is added.
        }
        if hover.hovered() {
            picking_events.send(PickingEvent::Hover(HoverEvent::JustEntered(entity)));
        } else {
            picking_events.send(PickingEvent::Hover(HoverEvent::JustLeft(entity)));
        }
    }
    for (entity, selection, selection_change) in selection_query.iter() {
        if selection_change.is_added() {
            continue; // Avoid a false change detection when a component is added.
        }
        if selection.selected() {
            picking_events.send(PickingEvent::Selection(SelectionEvent::JustSelected(
                entity,
            )));
        } else {
            picking_events.send(PickingEvent::Selection(SelectionEvent::JustDeselected(
                entity,
            )));
        }
    }

    for hit in cursors
        .iter()
        .filter_map(|(cursor, hit)| (cursor.enabled && cursor.clicked).then(|| hit))
    {
        for entity in &hit.entities {
            if hover_query
                .get_component::<Hover>(*entity)
                .map_or(false, |h| h.hovered())
            {
                picking_events.send(PickingEvent::Clicked(*entity));
            }
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system(
    mut events: EventReader<PickingEvent>,
    input_cursors: Query<&CursorInput, Changed<CursorInput>>,
    hit_cursors: Query<&CursorHit, Changed<CursorHit>>,
) {
    for event in events.iter() {
        info!("{:?}", event);
    }
    for cursor in input_cursors.iter() {
        info!(
            "CursorInput: ({:>6.1}, {:>6.1}, click: {})",
            cursor.position.x, cursor.position.y, cursor.clicked
        );
    }
    for hit in hit_cursors.iter() {
        info!("CursorHit: ({:?})", hit.entities);
    }
}
