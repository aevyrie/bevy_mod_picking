//! Debug tools for picking events

use bevy::prelude::*;

use crate::events::{IsPointerEvent, Pointer};

/// Listens for pointer events of type `E` and prints them.
pub fn print<E: IsPointerEvent + 'static>(mut pointer_events: EventReader<Pointer<E>>) {
    for event in pointer_events.iter() {
        info!("{event}");
    }
}

/// Tracks frame number for diagnostics.
#[derive(Debug, Default, Clone, Copy, Resource)]
pub struct Frame(pub usize);

/// Increments frame number for diagnostics.
pub fn increment_frame(mut frame: ResMut<Frame>) {
    frame.0 += 1;
}
