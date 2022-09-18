//! Debug tools for picking events

use bevy::prelude::*;

use crate::output::IsPointerEvent;

/// Listens for pointer events of type `E` and prints them.
pub fn print<E: IsPointerEvent>(mut events: EventReader<E>) {
    for event in events.iter() {
        info!("{event}, Event: {:?}", event.event());
    }
}
