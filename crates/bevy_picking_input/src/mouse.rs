use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
};
use bevy_picking_core::{
    input::{Location, PointerClickEvent, PointerLocationEvent},
    PointerId,
};

/// Sends mouse pointer events to be processed by the picking backend
pub fn mouse_pick_events(
    windows: Res<Windows>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    mut pointer_moves: EventWriter<PointerLocationEvent>,
    mut pointer_clicks: EventWriter<PointerClickEvent>,
) {
    let id = PointerId::Mouse;
    let location = match get_cursor_position(windows) {
        Some(location) => location,
        None => return,
    };
    pointer_moves.send(PointerLocationEvent { id, location });

    for input in mouse_inputs.iter() {
        if matches!(input.button, MouseButton::Left) {
            match input.state {
                ButtonState::Pressed => pointer_clicks.send(PointerClickEvent::Down {
                    id: PointerId::Mouse,
                }),
                ButtonState::Released => pointer_clicks.send(PointerClickEvent::Up {
                    id: PointerId::Mouse,
                }),
            }
        }
    }
}

fn get_cursor_position(windows: Res<Windows>) -> Option<Location> {
    for window in windows.iter() {
        if let Some(position) = window.cursor_position() {
            return Some(Location {
                position,
                target: RenderTarget::Window(window.id()),
            });
        }
    }
    None
}
