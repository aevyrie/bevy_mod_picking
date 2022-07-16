use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
};
use bevy_picking_core::{
    input::{Location, PointerButton, PointerLocationEvent, PointerPressEvent},
    PointerId,
};

/// Sends mouse pointer events to be processed by the picking backend
pub fn mouse_pick_events(
    windows: Res<Windows>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    mut pointer_moves: EventWriter<PointerLocationEvent>,
    mut pointer_clicks: EventWriter<PointerPressEvent>,
) {
    let id = PointerId::Mouse;
    let location = match get_cursor_position(windows) {
        Some(location) => location,
        None => return,
    };
    pointer_moves.send(PointerLocationEvent { id, location });

    for input in mouse_inputs.iter() {
        let button = match input.button {
            MouseButton::Left => PointerButton::Primary,
            MouseButton::Right => PointerButton::Secondary,
            MouseButton::Middle => PointerButton::Middle,
            MouseButton::Other(_) => continue,
        };

        match input.state {
            ButtonState::Pressed => {
                pointer_clicks.send(PointerPressEvent::new_down(PointerId::Mouse, button))
            }
            ButtonState::Released => {
                pointer_clicks.send(PointerPressEvent::new_up(PointerId::Mouse, button))
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
