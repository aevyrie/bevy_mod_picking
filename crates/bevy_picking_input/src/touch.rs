//! Provides sensible defaults for touch picking inputs.

use bevy::{
    input::touch::TouchPhase,
    prelude::*,
    render::camera::RenderTarget,
    utils::{HashMap, HashSet},
    window::{PrimaryWindow, WindowRef},
};
use bevy_picking_core::{
    events::PointerCancel,
    pointer::{InputMove, InputPress, Location, PointerButton, PointerId},
    PointerCoreBundle,
};

/// Sends touch pointer events to be consumed by the core plugin
///
/// IMPORTANT: the commands must be flushed after this system is run because we need spawning to
/// happen immediately to prevent issues with missed events needed for drag and drop.
pub fn touch_pick_events(
    // Input
    mut touches: EventReader<TouchInput>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    // Local
    mut location_cache: Local<HashMap<u64, TouchInput>>,
    // Output
    mut commands: Commands,
    mut input_moves: EventWriter<InputMove>,
    mut input_presses: EventWriter<InputPress>,
    mut cancel_events: EventWriter<PointerCancel>,
) {
    for touch in touches.iter() {
        let pointer = PointerId::Touch(touch.id);
        let location = Location {
            target: RenderTarget::Window(WindowRef::Primary)
                .normalize(Some(windows.single().0))
                .unwrap(),
            position: touch.position,
        };
        match touch.phase {
            TouchPhase::Started => {
                info!("Spawning pointer {:?}", pointer);
                commands.spawn((
                    PointerCoreBundle::new(pointer).with_location(location.clone()),
                    #[cfg(feature = "selection")]
                    bevy_picking_selection::PointerMultiselect::default(),
                ));

                input_moves.send(InputMove::new(pointer, location, Vec2::ZERO));
                input_presses.send(InputPress::new_down(pointer, PointerButton::Primary));
                location_cache.insert(touch.id, *touch);
            }
            TouchPhase::Moved => {
                // Send a move event only if it isn't the same as the last one
                if let Some(last_touch) = location_cache.get(&touch.id) {
                    if last_touch == touch {
                        break;
                    }
                    input_moves.send(InputMove::new(
                        pointer,
                        location,
                        touch.position - last_touch.position,
                    ));
                }
                location_cache.insert(touch.id, *touch);
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                input_presses.send(InputPress::new_up(pointer, PointerButton::Primary));
                location_cache.remove(&touch.id);
                cancel_events.send(PointerCancel {
                    pointer_id: pointer,
                })
            }
        }
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
            TouchPhase::Ended | TouchPhase::Canceled => {
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
        debug!("Despawning pointer {:?}", pointer);
        commands.entity(entity).despawn_recursive();
    }
}
