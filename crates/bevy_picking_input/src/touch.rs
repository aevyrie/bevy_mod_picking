//! Provides sensible defaults for touch picking inputs.

use bevy::{
    input::touch::TouchPhase,
    prelude::*,
    render::camera::RenderTarget,
    utils::{HashMap, HashSet},
    window::WindowId,
};
use bevy_picking_core::{
    output::PointerCancel,
    pointer::{InputMove, InputPress, Location, PointerButton, PointerId},
    PointerCoreBundle,
};

/// Sends touch pointer events to be consumed by the core plugin
pub fn touch_pick_events(
    // Input
    mut touches: EventReader<TouchInput>,
    windows: Res<Windows>,
    // Local
    mut location_cache: Local<HashMap<u64, TouchInput>>,
    // Output
    mut input_moves: EventWriter<InputMove>,
    mut input_presses: EventWriter<InputPress>,
    mut cancel_events: EventWriter<PointerCancel>,
) {
    let active_window = windows
        .iter()
        .filter_map(|window| window.is_focused().then_some(window.id()))
        .next()
        .unwrap_or_else(WindowId::primary);

    for touch in touches.iter() {
        match touch.phase {
            TouchPhase::Started => {
                let pointer = PointerId::Touch(touch.id);
                input_presses.send(InputPress::new_down(pointer, PointerButton::Primary));
                location_cache.insert(touch.id, *touch);
            }
            TouchPhase::Moved => {
                let pointer = PointerId::Touch(touch.id);
                let pos = touch.position;
                let height = windows.primary().height();
                let location = Location {
                    target: RenderTarget::Window(active_window),
                    position: Vec2::new(pos.x, height - pos.y),
                };
                // Send a move event only if it isn't the same as the last one
                if location_cache.get(&touch.id) != Some(touch) {
                    input_moves.send(InputMove::new(pointer, location));
                }
                location_cache.insert(touch.id, *touch);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let pointer = PointerId::Touch(touch.id);
                input_presses.send(InputPress::new_up(pointer, PointerButton::Primary));
                location_cache.remove(&touch.id);
                cancel_events.send(PointerCancel {
                    pointer_id: pointer,
                })
            }
        }
    }
}

/// Activates new touch pointers.
///
/// Care must be taken to ensure pointers are spawned without causing a stage delay.
pub fn activate_pointers(mut commands: Commands, mut touches: EventReader<TouchInput>) {
    for pointer_bundle in touches.iter().filter_map(|touch| {
        touch
            .phase
            .eq(&TouchPhase::Started)
            .then_some(PointerCoreBundle::new(PointerId::Touch(touch.id)))
    }) {
        info!("Spawning pointer {:?}", pointer_bundle.id);
        commands.spawn_bundle(pointer_bundle);
    }
}

/// Deactivates unused touch pointers.
///
/// Because each new touch gets assigned a new ID, we need to remove the pointers associated with
/// touches that are no longer active.
pub fn deactivate_pointers(
    mut commands: Commands,
    mut despawn_list: Local<HashSet<(Entity, PointerId)>>,
    pointers: Query<(Entity, &PointerId)>,
    mut touches: EventReader<TouchInput>,
) {
    for touch in touches.iter() {
        match touch.phase {
            TouchPhase::Ended | TouchPhase::Cancelled => {
                for (entity, pointer) in &pointers {
                    if pointer.get_touch_id() == Some(touch.id) {
                        despawn_list.insert((entity, *pointer));
                    }
                }
            }
            _ => {}
        }
    }
    // A hash set is used to prevent despawning the same entity twice.
    for (entity, pointer) in despawn_list.drain() {
        info!("Despawning pointer {:?}", pointer);
        commands.entity(entity).despawn_recursive();
    }
}
