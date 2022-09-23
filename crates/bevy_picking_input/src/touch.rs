//! Provides sensible defaults for touch picking inputs.

use bevy::{prelude::*, render::camera::RenderTarget, window::WindowId};
use bevy_picking_core::{
    pointer::{InputMove, InputPress, Location, PointerButton, PointerId},
    PointerBundle,
};

/// Sends touch pointer events to be consumed by the core plugin
pub fn touch_pick_events(
    // Input
    touches: Res<Touches>,
    windows: Res<Windows>,
    // Output
    mut input_moves: EventWriter<InputMove>,
    mut input_presses: EventWriter<InputPress>,
) {
    let active_window = windows
        .iter()
        .filter_map(|window| window.is_focused().then_some(window.id()))
        .next()
        .unwrap_or_else(WindowId::primary);

    for touch in touches.iter_just_pressed() {
        let pointer = PointerId::Touch(touch.id());
        input_presses.send(InputPress::new_down(pointer, PointerButton::Primary));
    }
    for touch in touches.iter() {
        let pointer = PointerId::Touch(touch.id());
        if touch.delta() != Vec2::ZERO {
            let pos = touch.position();
            let height = windows.primary().height();
            let location = Location {
                target: RenderTarget::Window(active_window),
                position: Vec2::new(pos.x, height - pos.y),
            };
            input_moves.send(InputMove::new(pointer, location))
        }
    }
    for touch in touches
        .iter_just_released()
        .chain(touches.iter_just_cancelled())
    {
        let pointer = PointerId::Touch(touch.id());
        input_presses.send(InputPress::new_up(pointer, PointerButton::Primary));
    }
}

/// Activates new touch pointers
pub fn activate_pointers(
    mut commands: Commands,
    mut pointers: Query<&mut PointerId>,
    touches: Res<Touches>,
) {
    let mut new_pointers: Vec<_> = touches.iter_just_pressed().collect();

    pointers
        .iter_mut()
        .filter(|p| **p == PointerId::Inactive)
        .for_each(|mut p| {
            if let Some(t) = new_pointers.pop() {
                *p = PointerId::Touch(t.id());
            }
        });

    for touch in new_pointers.drain(..) {
        warn!(
            "Spawning a touch pointer. This can result in a missed pointer down event.
        Ensure there are enough inactive pointers spawned at startup."
        );
        commands.spawn_bundle(PointerBundle::new(PointerId::Touch(touch.id())));
    }
}

/// Deactivates unused touch pointers.
///
/// Because each new touch gets assigned a new ID, we need to remove the pointers associated with
/// touches that are no longer active.
pub fn deactivate_pointers(
    mut pointers: Query<&mut PointerId>,
    mut inactive_pointers: Local<Vec<u64>>,
    touches: Res<Touches>,
) {
    // Deactivate touch pointers that were released or cancelled *last* frame.
    for mut pointer in &mut pointers {
        if pointer
            .get_touch_id()
            .iter()
            .any(|id| inactive_pointers.contains(id))
        {
            *pointer = PointerId::Inactive;
        }
    }

    inactive_pointers.clear();

    touches
        .iter_just_released()
        .chain(touches.iter_just_cancelled())
        .for_each(|touch| inactive_pointers.push(touch.id()));
}
