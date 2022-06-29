use crate::{
    hit::CursorHit,
    input::{CursorClick, CursorInput},
    Selection,
};
use bevy::{prelude::*, ui::FocusPolicy};

/// An event that triggers when the selection state of a [Selection] enabled [PickableTarget] changes.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub enum SelectEvent {
    JustSelected(Entity),
    JustDeselected(Entity),
}

/// An event that triggers when the hover state of a [Hover] enabled [PickableTarget] changes.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub enum HoverEvent {
    JustEntered(Entity),
    JustLeft(Entity),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub enum ClickEvent {
    JustClicked(Entity),
    JustReleased(Entity),
}

/// An event that wraps selection and hover events
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub enum PickingEvent {
    Select(SelectEvent),
    Hover(HoverEvent),
    Click(ClickEvent),
}

/// Looks for changes in selection or hover state, and sends the appropriate events
#[allow(clippy::type_complexity)]
pub fn write_events(
    cursors: Query<(&CursorClick, ChangeTrackers<CursorClick>, &CursorHit)>,
    focus: Query<&FocusPolicy>,
    selection_query: Query<(Entity, &Selection, ChangeTrackers<Selection>), Changed<Selection>>,
    mut events: EventWriter<PickingEvent>,
) {
    for (click, click_track, hit) in cursors.iter() {
        // TODO: handle conflicting cursor interactions. e.g. if two cursors attempt to modify the
        // interaction state of a target entity, which one takes precedence?
        for entity in hit.entities.iter() {
            if click.clicked && click_track.is_changed() {
                events.send(PickingEvent::Click(ClickEvent::JustClicked(*entity)));
            } else if !click.clicked && click_track.is_changed() {
                events.send(PickingEvent::Click(ClickEvent::JustReleased(*entity)));
            }

            if let Ok(_policy @ FocusPolicy::Block) = focus.get(*entity) {
                break; // Prevents interacting with anything further away
            }
        }
    }

    for (entity, selection, selection_change) in selection_query.iter() {
        if selection_change.is_added() {
            continue; // Avoid a false change detection when a component is added.
        }
        if selection.selected() {
            events.send(PickingEvent::Select(SelectEvent::JustSelected(entity)));
        } else {
            events.send(PickingEvent::Select(SelectEvent::JustDeselected(entity)));
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system(
    mut events: EventReader<PickingEvent>,
    input_cursors: Query<&CursorInput, Changed<CursorInput>>,
    hit_cursors: Query<&CursorHit, Changed<CursorHit>>,
    click_cursors: Query<&CursorClick, Changed<CursorClick>>,
) {
    for event in events.iter() {
        info!("Event: {:?}", event);
    }
    for cursor in input_cursors.iter() {
        info!(
            "CursorInput: ( {:>6.1}, {:>6.1} )",
            cursor.position.x, cursor.position.y
        );
    }
    for hit in hit_cursors.iter() {
        info!("CursorHit: ( {:?} )", hit.entities);
    }
    for click in click_cursors.iter() {
        info!("CursorClick: ( {:?} )", click.clicked);
    }
}
