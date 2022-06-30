use crate::{
    backend::CursorOver,
    input::{CursorClick, CursorId, CursorLocation},
};
use bevy::prelude::*;

/// An event that wraps selection and hover events
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub enum Just {
    /// Like `mouseover`    
    Entered,
    /// Like `mouseout`
    Exited,
    /// Like `mousedown`
    Down,
    /// Like `mouseup`
    Up,
    /// Like `click`
    Click,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct CursorEvent {
    pub entity: Entity,
    pub cursor: CursorId,
    pub event: Just,
}

impl CursorEvent {
    pub fn new(entity: Entity, cursor: CursorId, event: Just) -> Self {
        Self {
            entity,
            cursor,
            event,
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system(
    mut events: EventReader<CursorEvent>,
    location_cursors: Query<&CursorLocation, Changed<CursorLocation>>,
    hit_cursors: Query<&CursorOver, Changed<CursorOver>>,
    click_cursors: Query<&CursorClick, Changed<CursorClick>>,
) {
    for location in location_cursors.iter() {
        info!("CursorLocation: ( {:?} )", location);
    }
    for click in click_cursors.iter() {
        info!("CursorClick: ( {:?} )", click.is_clicked);
    }
    for hit in hit_cursors.iter() {
        info!("CursorOver: ( {:?} )", hit.entities);
    }
    for event in events.iter() {
        info!("Event: {:?}", event);
    }
}
