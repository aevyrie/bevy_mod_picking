use bevy::{prelude::*, render::camera::RenderTarget, window::WindowId};
use bevy_picking_core::{
    input::{InputMove, InputPress, Location, PointerButton, PressStage},
    PointerId,
};

/// Sends touch pointer events to be consumed by the core plugin
pub fn touch_pick_events(
    touches: Res<Touches>,
    mut pointer_moves: EventWriter<InputMove>,
    mut pointer_clicks: EventWriter<InputPress>,
) {
    for touch in touches.iter() {
        if touch.distance() != Vec2::ZERO {
            let id = PointerId::Touch(touch.id());
            let location = Location {
                target: RenderTarget::Window(WindowId::primary()),
                position: touch.position(),
            };
            pointer_moves.send(InputMove { id, location })
        }
    }
    for touch in touches.iter_just_pressed() {
        pointer_clicks.send(InputPress {
            id: PointerId::Touch(touch.id()),
            press: PressStage::Down,
            button: PointerButton::Primary,
        })
    }
    for touch in touches.iter_just_released() {
        pointer_clicks.send(InputPress {
            id: PointerId::Touch(touch.id()),
            press: PressStage::Up,
            button: PointerButton::Primary,
        })
    }
}
