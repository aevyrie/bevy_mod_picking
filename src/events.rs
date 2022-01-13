use crate::{Hover, PickableMesh, Selection};
use bevy::prelude::*;

/// An event that triggers when the selection state of a [Selection] enabled [PickableMesh] changes.
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

/// An event that wraps selection and hover events
#[derive(Debug)]
pub enum PickingEvent {
    Selection(SelectionEvent),
    Hover(HoverEvent),
    Clicked(Entity),
}

/// Looks for changes in selection or hover state, and sends the appropriate events
#[allow(clippy::type_complexity)]
pub fn mesh_events_system(
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    mut picking_events: EventWriter<PickingEvent>,
    hover_query: Query<
        (Entity, &Hover, ChangeTrackers<Hover>),
        (Changed<Hover>, With<PickableMesh>),
    >,
    selection_query: Query<
        (Entity, &Selection, ChangeTrackers<Selection>),
        (Changed<Selection>, With<PickableMesh>),
    >,
    click_query: Query<(Entity, &Hover)>,
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
    if mouse_button_input.just_pressed(MouseButton::Left)
        || touches_input.iter_just_pressed().next().is_some()
    {
        for (entity, hover) in click_query.iter() {
            if hover.hovered() {
                picking_events.send(PickingEvent::Clicked(entity));
            }
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        info!("{:?}", event);
    }
}
