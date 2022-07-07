use bevy::{prelude::*, render::camera::RenderTarget, window::WindowId};
use bevy_picking_core::{
    input::{Location, PointerClickEvent, PointerLocationEvent},
    PointerId,
};

/// Sends touch pointer events to be processed by the picking backend
pub fn touch_pick_events(
    touches: Res<Touches>,
    mut pointer_moves: EventWriter<PointerLocationEvent>,
    mut pointer_clicks: EventWriter<PointerClickEvent>,
) {
    for touch in touches.iter() {
        let id = PointerId::Touch(touch.id());
        let location = Location {
            target: RenderTarget::Window(WindowId::primary()),
            position: touch.position(),
        };
        pointer_moves.send(PointerLocationEvent { id, location })
    }
    for touch in touches.iter_just_pressed() {
        pointer_clicks.send(PointerClickEvent::Down {
            id: PointerId::Touch(touch.id()),
        })
    }
    for touch in touches.iter_just_released() {
        pointer_clicks.send(PointerClickEvent::Up {
            id: PointerId::Touch(touch.id()),
        })
    }
}
