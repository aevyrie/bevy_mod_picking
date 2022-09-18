//! Debug tools for picking inputs

use bevy::prelude::*;
use bevy_picking_core::pointer::{InputMove, InputPress};

/// Listens for input events and prints them.
pub fn print(mut moves: EventReader<InputMove>, mut presses: EventReader<InputPress>) {
    for event in moves.iter() {
        info!("Input Move: {:?}", event.id());
    }
    for event in presses.iter() {
        info!("Input Press: {:?}, {:?}", event.id(), event.press());
    }
}
