use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
};
use bevy_picking_core::{
    input::{InputMove, InputPress, Location, PointerButton},
    PointerId,
};

/// Sends mouse pointer events to be processed by the core plugin
pub fn mouse_pick_events(
    // Input
    mut cursor_moves: EventReader<CursorMoved>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    // Output
    mut pointer_move: EventWriter<InputMove>,
    mut pointer_clicks: EventWriter<InputPress>,
) {
    for event in cursor_moves.iter() {
        pointer_move.send(InputMove {
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
                pointer_clicks.send(InputPress::new_down(PointerId::Mouse, button))
            }
            ButtonState::Released => {
                pointer_clicks.send(InputPress::new_up(PointerId::Mouse, button))
            }
        }
    }
}
