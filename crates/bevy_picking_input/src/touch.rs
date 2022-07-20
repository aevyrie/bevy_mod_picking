use bevy::{prelude::*, render::camera::RenderTarget, window::WindowId};
use bevy_picking_core::{
    input::{Location, PointerButton, PointerMoveEvent, PointerPressEvent, PressStage},
    PointerId,
};

/// Sends touch pointer events to be processed by the picking backend
pub fn touch_pick_events(
    touches: Res<Touches>,
    mut pointer_moves: EventWriter<PointerMoveEvent>,
    mut pointer_clicks: EventWriter<PointerPressEvent>,
) {
    for touch in touches.iter() {
        if touch.distance() != Vec2::ZERO {
            let id = PointerId::Touch(touch.id());
            let location = Location {
                target: RenderTarget::Window(WindowId::primary()),
                position: touch.position(),
            };
            pointer_moves.send(PointerMoveEvent { id, location })
        }
    }
    for touch in touches.iter_just_pressed() {
        pointer_clicks.send(PointerPressEvent {
            id: PointerId::Touch(touch.id()),
            press: PressStage::Down,
            button: PointerButton::Primary,
        })
    }
    for touch in touches.iter_just_released() {
        pointer_clicks.send(PointerPressEvent {
            id: PointerId::Touch(touch.id()),
            press: PressStage::Up,
            button: PointerButton::Primary,
        })
    }
}
