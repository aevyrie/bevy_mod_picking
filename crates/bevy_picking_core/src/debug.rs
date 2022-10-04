//! Debug tools for picking events

use bevy::prelude::*;

use crate::output::IsPointerEvent;

/// Listens for pointer events of type `E` and prints them.
pub fn print<E: IsPointerEvent>(mut pointer_events: EventReader<E>) {
    for event in pointer_events.iter() {
        info!("Pointer Event: {:?}, {event}", event.event_type());
    }
}
