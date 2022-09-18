//! Provides sensible defaults for touch picking inputs.

use bevy::{prelude::*, render::camera::RenderTarget, utils::HashSet, window::WindowId};
use bevy_picking_core::pointer::{InputMove, InputPress, Location, PointerButton, PointerId};

/// Sends touch pointer events to be consumed by the core plugin
pub fn touch_pick_events(
    touches: Res<Touches>,
    windows: Res<Windows>,
    pointers: Query<(Entity, &PointerId)>,
    mut commands: Commands,
    mut pointer_moves: EventWriter<InputMove>,
    mut pointer_clicks: EventWriter<InputPress>,
) {
    let mut active_pointers = HashSet::new();
    let active_window = windows
        .iter()
        .filter_map(|window| window.is_focused().then_some(window.id()))
        .next()
        .unwrap_or(WindowId::primary());

    for touch in touches.iter() {
        let pointer = PointerId::Touch(touch.id());
        active_pointers.insert(pointer);
        if touch.delta() != Vec2::ZERO {
            let pos = touch.position();
            let height = windows.primary().height();
            let location = Location {
                target: RenderTarget::Window(active_window),
                position: Vec2::new(pos.x, height - pos.y),
            };
            pointer_moves.send(InputMove::new(pointer, location))
        }
    }
    for touch in touches.iter_just_pressed() {
        let pointer = PointerId::Touch(touch.id());
        pointer_clicks.send(InputPress::new_down(pointer, PointerButton::Primary));
    }
    for touch in touches
        .iter_just_released()
        .chain(touches.iter_just_cancelled())
    {
        let pointer = PointerId::Touch(touch.id());
        active_pointers.insert(pointer);
        pointer_clicks.send(InputPress::new_up(pointer, PointerButton::Primary));
    }

    // Because each new touch gets assigned a new ID, we need to remove the touches that are no
    // longer active.
    for (entity, pointer) in pointers
        .iter()
        .filter(|(_, id)| matches!(id, PointerId::Touch(_)))
    {
        if !active_pointers.contains(pointer) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
