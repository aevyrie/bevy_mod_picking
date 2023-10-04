//! Processes data from input and backends, producing interaction events.

use std::fmt::Debug;

use crate::{
    backend::HitData,
    focus::{HoverMap, PreviousHoverMap},
    pointer::{
        self, InputMove, InputPress, Location, PointerButton, PointerId, PointerLocation,
        PointerMap, PressDirection,
    },
};
use bevy::{prelude::*, utils::HashMap};
use bevy_eventlistener::prelude::*;

/// Stores the common data needed for all `PointerEvent`s.
#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct Pointer<E: Debug + Clone + Reflect> {
    /// The target of this event
    #[target]
    pub target: Entity,
    /// The pointer that triggered this event
    pub pointer_id: PointerId,
    /// The location of the pointer during this event
    pub pointer_location: Location,
    /// Additional event-specific data. [`Drop`] for example, has an additional field to describe
    /// the `Entity` that is being dropped on the target.
    pub event: E,
}

impl<E: Debug + Clone + Reflect> std::fmt::Display for Pointer<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:?}, {:.1?}, {:?}, {:.1?}",
            self.pointer_id, self.pointer_location.position, self.target, self.event
        ))
    }
}

impl<E: Debug + Clone + Reflect> std::ops::Deref for Pointer<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<E: Debug + Clone + Reflect> Pointer<E> {
    /// Construct a new `PointerEvent`.
    pub fn new(id: PointerId, location: Location, target: Entity, event: E) -> Self {
        Self {
            pointer_id: id,
            pointer_location: location,
            target,
            event,
        }
    }
}

/// Fires when a pointer is no longer available.
#[derive(Event, Clone, PartialEq, Debug, Reflect)]
pub struct PointerCancel {
    /// ID of the pointer that was cancelled.
    #[reflect(ignore)]
    pub pointer_id: PointerId,
}

/// Fires when a the pointer crosses into the bounds of the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Over {
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires when a the pointer crosses out of the bounds of the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Out {
    /// Information about the latest prior picking intersection.
    pub hit: HitData,
}

/// Fires when a pointer button is pressed over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Down {
    /// Pointer button pressed to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires when a pointer button is released over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Up {
    /// Pointer button lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires when a pointer sends a pointer down event followed by a pointer up event, with the same
/// `target` entity for both events.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Click {
    /// Pointer button pressed and lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires while a pointer is moving over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Move {
    /// Information about the picking intersection.
    pub hit: HitData,
    /// The change in position since the last move event.
    pub delta: Vec2,
}

/// Fires when the `target` entity receives a pointer down event followed by a pointer move event.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragStart {
    /// Pointer button pressed and moved to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires while the `target` entity is being dragged.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Drag {
    /// Pointer button pressed and moved to trigger this event.
    pub button: PointerButton,
    /// The total distance vector of a drag, measured from drag start to the current position.
    pub distance: Vec2,
    /// The change in position since the last drag event.
    pub delta: Vec2,
}

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragEnd {
    /// Pointer button pressed, moved, and lifted to trigger this event.
    pub button: PointerButton,
    /// The vector of drag movement measured from start to final pointer position.
    pub distance: Vec2,
}

/// Fires when a pointer dragging the `dragged` entity enters the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragEnter {
    /// Pointer button pressed to enter drag.
    pub button: PointerButton,
    /// The entity that was being dragged when the pointer entered the `target` entity.
    pub dragged: Entity,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires while the `dragged` entity is being dragged over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragOver {
    /// Pointer button pressed while dragging over.
    pub button: PointerButton,
    /// The entity that was being dragged when the pointer was over the `target` entity.
    pub dragged: Entity,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Fires when a pointer dragging the `dragged` entity leaves the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragLeave {
    /// Pointer button pressed while leaving drag.
    pub button: PointerButton,
    /// The entity that was being dragged when the pointer left the `target` entity.
    pub dragged: Entity,
    /// Information about the latest prior picking intersection.
    pub hit: HitData,
}

/// Fires when a pointer drops the `dropped` entity onto the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Drop {
    /// Pointer button lifted to drop.
    pub button: PointerButton,
    /// The entity that was dropped onto the `target` entity.
    pub dropped: Entity,
    /// Information about the picking intersection.
    pub hit: HitData,
}

/// Generates pointer events from input data
pub fn pointer_events(
    // Input
    mut input_presses: EventReader<InputPress>,
    mut input_moves: EventReader<pointer::InputMove>,
    pointer_map: Res<PointerMap>,
    pointers: Query<&PointerLocation>,
    hover_map: Res<HoverMap>,
    previous_hover_map: Res<PreviousHoverMap>,
    // Output
    mut pointer_move: EventWriter<Pointer<Move>>,
    mut pointer_over: EventWriter<Pointer<Over>>,
    mut pointer_out: EventWriter<Pointer<Out>>,
    mut pointer_up: EventWriter<Pointer<Up>>,
    mut pointer_down: EventWriter<Pointer<Down>>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    for InputMove {
        pointer_id,
        location,
        delta,
    } in input_moves.iter().cloned()
    {
        for (hovered_entity, hit) in hover_map
            .get(&pointer_id)
            .iter()
            .flat_map(|h| h.iter().map(|(entity, data)| (*entity, data.to_owned())))
        {
            pointer_move.send(Pointer::new(
                pointer_id,
                location.clone(),
                hovered_entity,
                Move { hit, delta },
            ))
        }
    }

    for press_event in input_presses.iter() {
        let button = press_event.button;
        // We use the previous hover map because we want to consider pointers that just left the
        // entity. Without this, touch inputs would never send up events because they are lifted up
        // and leave the bounds of the entity at the same time.
        for (hovered_entity, hit) in previous_hover_map
            .get(&press_event.pointer_id)
            .iter()
            .flat_map(|h| h.iter().map(|(entity, data)| (*entity, data.clone())))
        {
            if let PressDirection::Up = press_event.direction {
                let Some(location) = pointer_location(press_event.pointer_id) else {
                    error!(
                        "Unable to get location for pointer {:?}",
                        press_event.pointer_id
                    );
                    continue;
                };
                pointer_up.send(Pointer::new(
                    press_event.pointer_id,
                    location,
                    hovered_entity,
                    Up { button, hit },
                ))
            }
        }
        for (hovered_entity, hit) in hover_map
            .get(&press_event.pointer_id)
            .iter()
            .flat_map(|h| h.iter().map(|(entity, data)| (*entity, data.clone())))
        {
            if let PressDirection::Down = press_event.direction {
                let Some(location) = pointer_location(press_event.pointer_id) else {
                    error!(
                        "Unable to get location for pointer {:?}",
                        press_event.pointer_id
                    );
                    continue;
                };
                pointer_down.send(Pointer::new(
                    press_event.pointer_id,
                    location,
                    hovered_entity,
                    Down { button, hit },
                ))
            }
        }
    }

    // If the entity is hovered...
    for (pointer_id, hovered_entity, hit) in hover_map
        .iter()
        .flat_map(|(id, hashmap)| hashmap.iter().map(|data| (*id, *data.0, data.1.clone())))
    {
        // ...but was not hovered last frame...
        if !previous_hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains_key(&hovered_entity))
        {
            let Some(location) = pointer_location(pointer_id) else {
                error!("Unable to get location for pointer {:?}", pointer_id);
                continue;
            };
            pointer_over.send(Pointer::new(
                pointer_id,
                location,
                hovered_entity,
                Over { hit },
            ));
        }
    }

    // If the entity was hovered by a specific pointer last frame...
    for (pointer_id, hovered_entity, hit) in previous_hover_map
        .iter()
        .flat_map(|(id, hashmap)| hashmap.iter().map(|data| (*id, *data.0, data.1.clone())))
    {
        // ...but is now not being hovered by that same pointer...
        if !hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains_key(&hovered_entity))
        {
            let Some(location) = pointer_location(pointer_id) else {
                error!("Unable to get location for pointer {:?}", pointer_id);
                continue;
            };
            pointer_out.send(Pointer::new(
                pointer_id,
                location,
                hovered_entity,
                Out { hit },
            ));
        }
    }
}

/// Maps pointers to the entities they are dragging.
#[derive(Debug, Deref, DerefMut, Default, Resource)]
pub struct DragMap(pub HashMap<(PointerId, PointerButton), HashMap<Entity, DragEntry>>);

/// An entry in the [`DragMap`].
#[derive(Debug, Clone)]
pub struct DragEntry {
    /// The position of the pointer at drag start.
    pub start_pos: Vec2,
    /// The latest position of the pointer during this drag, used to compute deltas.
    pub latest_pos: Vec2,
}

/// Uses pointer events to determine when click and drag events occur.
pub fn send_click_and_drag_events(
    // Input
    mut pointer_down: EventReader<Pointer<Down>>,
    mut pointer_up: EventReader<Pointer<Up>>,
    mut input_move: EventReader<InputMove>,
    mut input_presses: EventReader<InputPress>,
    pointer_map: Res<PointerMap>,
    pointers: Query<&PointerLocation>,
    // Locals
    mut down_map: Local<HashMap<(PointerId, PointerButton), HashMap<Entity, Pointer<Down>>>>,
    // Output
    mut drag_map: ResMut<DragMap>,
    mut pointer_click: EventWriter<Pointer<Click>>,
    mut pointer_drag_start: EventWriter<Pointer<DragStart>>,
    mut pointer_drag_end: EventWriter<Pointer<DragEnd>>,
    mut pointer_drag: EventWriter<Pointer<Drag>>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    // Triggers during movement even if not over an entity
    for InputMove {
        pointer_id,
        location,
        delta: _,
    } in input_move.iter().cloned()
    {
        for button in PointerButton::iter() {
            let Some(down_list) = down_map.get(&(pointer_id, button)) else {
                continue;
            };
            let drag_list = drag_map.entry((pointer_id, button)).or_default();

            for down in down_list.values() {
                if drag_list.contains_key(&down.target) {
                    continue; // this entity is already logged as being dragged
                }
                drag_list.insert(
                    down.target,
                    DragEntry {
                        start_pos: down.pointer_location.position,
                        latest_pos: down.pointer_location.position,
                    },
                );
                pointer_drag_start.send(Pointer::new(
                    pointer_id,
                    down.pointer_location.clone(),
                    down.target,
                    DragStart {
                        button,
                        hit: down.hit.clone(),
                    },
                ))
            }

            for (dragged_entity, drag) in drag_list.iter_mut() {
                let drag_event = Drag {
                    button,
                    distance: location.position - drag.start_pos,
                    delta: location.position - drag.latest_pos,
                };
                drag.latest_pos = location.position;
                pointer_drag.send(Pointer::new(
                    pointer_id,
                    location.clone(),
                    *dragged_entity,
                    drag_event,
                ))
            }
        }
    }

    // Triggers when button is released over an entity
    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: Up { button, hit },
    } in pointer_up.iter().cloned()
    {
        // Can't have a click without the button being pressed down first
        if down_map
            .get(&(pointer_id, button))
            .and_then(|down| down.get(&target))
            .is_some()
        {
            pointer_click.send(Pointer::new(
                pointer_id,
                pointer_location,
                target,
                Click { button, hit },
            ));
        }
    }

    // Triggers when button is pressed over an entity
    for event in pointer_down.iter() {
        let button = event.button;
        let down_button_entity_map = down_map.entry((event.pointer_id, button)).or_default();
        down_button_entity_map.insert(event.target, event.clone());
    }

    // Triggered for all button presses
    for press in input_presses.iter() {
        if press.direction != pointer::PressDirection::Up {
            continue; // We are only interested in button releases
        }
        down_map.insert((press.pointer_id, press.button), HashMap::new());
        let Some(drag_list) = drag_map.insert((press.pointer_id, press.button), HashMap::new())
        else {
            continue;
        };
        let Some(location) = pointer_location(press.pointer_id) else {
            error!("Unable to get location for pointer {:?}", press.pointer_id);
            continue;
        };

        for (drag_target, drag) in drag_list {
            let drag_end = DragEnd {
                button: press.button,
                distance: drag.latest_pos - drag.start_pos,
            };
            pointer_drag_end.send(Pointer::new(
                press.pointer_id,
                location.clone(),
                drag_target,
                drag_end,
            ));
        }
    }
}

/// Uses pointer events to determine when drag-over events occur
pub fn send_drag_over_events(
    // Input
    drag_map: Res<DragMap>,
    mut pointer_over: EventReader<Pointer<Over>>,
    mut pointer_move: EventReader<Pointer<Move>>,
    mut pointer_out: EventReader<Pointer<Out>>,
    mut pointer_drag_end: EventReader<Pointer<DragEnd>>,
    // Local
    mut drag_over_map: Local<HashMap<(PointerId, PointerButton), HashMap<Entity, HitData>>>,

    // Output
    mut pointer_drag_enter: EventWriter<Pointer<DragEnter>>,
    mut pointer_drag_over: EventWriter<Pointer<DragOver>>,
    mut pointer_drag_leave: EventWriter<Pointer<DragLeave>>,
    mut pointer_drop: EventWriter<Pointer<Drop>>,
) {
    // Fire PointerDragEnter events.
    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: Over { hit },
    } in pointer_over.iter().cloned()
    {
        for button in PointerButton::iter() {
            for drag_target in drag_map
                .get(&(pointer_id, button))
                .iter()
                .flat_map(|drag_list| drag_list.keys())
                .filter(
                    |&&drag_target| target != drag_target, /* can't drag over itself */
                )
            {
                let drag_entry = drag_over_map.entry((pointer_id, button)).or_default();
                drag_entry.insert(target, hit.clone());
                let event = DragEnter {
                    button,
                    dragged: *drag_target,
                    hit: hit.clone(),
                };
                pointer_drag_enter.send(Pointer::new(
                    pointer_id,
                    pointer_location.clone(),
                    target,
                    event,
                ))
            }
        }
    }

    // Fire PointerDragOver events.
    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: Move { hit, delta: _ },
    } in pointer_move.iter().cloned()
    {
        for button in PointerButton::iter() {
            for drag_target in drag_map
                .get(&(pointer_id, button))
                .iter()
                .flat_map(|drag_list| drag_list.keys())
                .filter(
                    |&&drag_target| target != drag_target, /* can't drag over itself */
                )
            {
                pointer_drag_over.send(Pointer::new(
                    pointer_id,
                    pointer_location.clone(),
                    target,
                    DragOver {
                        button,
                        dragged: *drag_target,
                        hit: hit.clone(),
                    },
                ))
            }
        }
    }

    // Fire PointerDragLeave and PointerDrop events when the pointer stops dragging.
    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: DragEnd {
            button,
            distance: _,
        },
    } in pointer_drag_end.iter().cloned()
    {
        let Some(drag_over_set) = drag_over_map.get_mut(&(pointer_id, button)) else {
            continue;
        };
        for (dragged_over, hit) in drag_over_set.drain() {
            pointer_drag_leave.send(Pointer::new(
                pointer_id,
                pointer_location.clone(),
                dragged_over,
                DragLeave {
                    button,
                    dragged: target,
                    hit: hit.clone(),
                },
            ));
            pointer_drop.send(Pointer::new(
                pointer_id,
                pointer_location.clone(),
                dragged_over,
                Drop {
                    button,
                    dropped: target,
                    hit: hit.clone(),
                },
            ));
        }
    }

    // Fire PointerDragLeave events when the pointer goes out of the target.
    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: Out { hit },
    } in pointer_out.iter().cloned()
    {
        for button in PointerButton::iter() {
            let Some(dragged_over) = drag_over_map.get_mut(&(pointer_id, button)) else {
                continue;
            };
            if dragged_over.remove(&target).is_none() {
                continue;
            }
            let Some(drag_list) = drag_map.get(&(pointer_id, button)) else {
                continue;
            };
            for drag_target in drag_list.keys() {
                pointer_drag_leave.send(Pointer::new(
                    pointer_id,
                    pointer_location.clone(),
                    target,
                    DragLeave {
                        button,
                        dragged: *drag_target,
                        hit: hit.clone(),
                    },
                ))
            }
        }
    }
}
