//! Provides sensible defaults for touch picking inputs.

use bevy::{
    ecs::system::{SystemParam, SystemState},
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

#[derive(SystemParam)]
struct TouchState<'w, 's> {
    // Input
    touches: EventReader<'w, 's, TouchInput>,
    windows: Res<'w, Windows>,
    // Local
    location_cache: Local<'s, HashMap<u64, TouchInput>>,
    // Output
    commands: Commands<'w, 's>,
    input_moves: EventWriter<'w, 's, InputMove>,
    input_presses: EventWriter<'w, 's, InputPress>,
    cancel_events: EventWriter<'w, 's, PointerCancel>,
}

#[derive(Resource, Deref, DerefMut)]
struct CachedSystemState<T: SystemParam + 'static>(SystemState<T>);

/// Sends touch pointer events to be consumed by the core plugin
///
/// This is an exclusive event because we need spawning to happen immediately to prevent issues with
/// missed events needed for drag and drop.
pub fn touch_pick_events(world: &mut World) {
    if world
        .get_resource::<CachedSystemState<TouchState>>()
        .is_none()
    {
        let state = SystemState::<TouchState>::new(world);
        world.insert_resource(CachedSystemState(state));
    }
    world.resource_scope(|world, mut state: Mut<CachedSystemState<TouchState>>| {
        let TouchState {
            // Input
            mut touches,
            windows,
            // Local
            mut location_cache,
            // Output
            mut commands,
            mut input_moves,
            mut input_presses,
            mut cancel_events,
        } = state.get_mut(world);

        let active_window = windows
            .iter()
            .filter_map(|window| window.is_focused().then_some(window.id()))
            .next()
            .unwrap_or_else(WindowId::primary);

        for touch in touches.iter() {
            let pointer = PointerId::Touch(touch.id);
            let pos = touch.position;
            let height = windows.primary().height();
            let location = Location {
                target: RenderTarget::Window(active_window),
                position: Vec2::new(pos.x, height - pos.y),
            };
            match touch.phase {
                TouchPhase::Started => {
                    info!("Spawning pointer {:?}", pointer);
                    commands.spawn((
                        PointerCoreBundle::new(pointer).with_location(location.clone()),
                        bevy_picking_selection::PointerMultiselect::default(),
                    ));

                    input_moves.send(InputMove::new(pointer, location));
                    input_presses.send(InputPress::new_down(pointer, PointerButton::Primary));
                    location_cache.insert(touch.id, *touch);
                }
                TouchPhase::Moved => {
                    // Send a move event only if it isn't the same as the last one
                    if location_cache.get(&touch.id) != Some(touch) {
                        input_moves.send(InputMove::new(pointer, location));
                    }
                    location_cache.insert(touch.id, *touch);
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    input_presses.send(InputPress::new_up(pointer, PointerButton::Primary));
                    location_cache.remove(&touch.id);
                    cancel_events.send(PointerCancel {
                        pointer_id: pointer,
                    })
                }
            }
        }

        state.apply(world);
    });
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
        debug!("Despawning pointer {:?}", pointer);
        commands.entity(entity).despawn_recursive();
    }
}
