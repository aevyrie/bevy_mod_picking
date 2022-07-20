use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
};
use bevy_picking_core::{
    input::{Location, PointerButton, PointerMoveEvent, PointerPressEvent},
    PointerId,
};

/// Sends mouse pointer events to be processed by the picking backend
pub fn mouse_pick_events(
    mut cursor_moves: EventReader<CursorMoved>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    mut pointer_move: EventWriter<PointerMoveEvent>,
    mut pointer_clicks: EventWriter<PointerPressEvent>,
) {
    for event in cursor_moves.iter() {
        pointer_move.send(PointerMoveEvent {
            id: PointerId::Mouse,
            location: Location {
                target: RenderTarget::Window(event.id),
                position: event.position,
            },
        });
    }

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
