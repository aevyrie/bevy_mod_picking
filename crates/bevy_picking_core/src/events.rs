//! Processes data from input and backends, producing interaction events.

use crate::{
    backend::HitData,
    focus::{HoverMap, PreviousHoverMap},
    pointer::{
        self, InputMove, InputPress, Location, PointerButton, PointerId, PointerLocation,
        PointerMap, PressDirection,
    },
};
use bevy::{ecs::event::Event, prelude::*, utils::HashMap};

/// Can be implemented on a custom event to allow [`EventListener`]s to convert [`PointerEvent`]s
/// into the custom event type.
pub trait ForwardedEvent<E: IsPointerEvent>: Event {
    /// Create a new event from [`EventListenerData`].
    fn from_data(event_data: &ListenedEvent<E>) -> Self;
}

/// An `EventListener` marks an entity, informing the [`event_bubbling`] system to run the
/// `callback` function when an event of type `E` is being bubbled up the hierarchy and reaches this
/// entity.
#[derive(Component, Clone, Reflect)]
pub struct EventListener<E: IsPointerEvent> {
    #[reflect(ignore)]
    /// A function that is called when the event listener is triggered.
    callback: fn(&mut Commands, &ListenedEvent<E>, &mut Bubble),
}

impl<E: IsPointerEvent> EventListener<E> {
    /// Create an [`EventListener`] that will run the supplied `callback` function with access to
    /// bevy [`Commands`] when the pointer event reaches this entity.
    pub fn callback(callback: fn(&mut Commands, &ListenedEvent<E>, &mut Bubble)) -> Self {
        Self { callback }
    }

    /// Create an [`EventListener`] that will send an event of type `F` when the listener is
    /// triggered, then continue to bubble the original event up this entity's hierarchy.
    pub fn forward_event<F: ForwardedEvent<E>>() -> Self {
        Self::callback(
            |commands: &mut Commands, event_data: &ListenedEvent<E>, _bubble: &mut Bubble| {
                let forwarded_event = F::from_data(event_data);
                commands.add(|world: &mut World| {
                    let mut events = world.get_resource_or_insert_with(Events::<F>::default);
                    events.send(forwarded_event);
                });
            },
        )
    }

    /// Create an [`EventListener`] that will send an event of type `F` when the listener is
    /// triggered, then halt bubbling, preventing event listeners of the same type from triggering
    /// on parents of this entity.
    ///
    /// Prefer using `new_forward_event` instead, unless you have a good reason to halt bubbling.
    pub fn forward_event_and_halt<F: ForwardedEvent<E>>() -> Self {
        Self {
            callback: |commands: &mut Commands,
                       event_data: &ListenedEvent<E>,
                       bubble: &mut Bubble| {
                let forwarded_event = F::from_data(event_data);
                commands.add(|world: &mut World| {
                    let mut events = world.get_resource_or_insert_with(Events::<F>::default);
                    events.send(forwarded_event);
                });
                bubble.burst();
            },
        }
    }
}

/// Data from a pointer event returned by an [`EventListener`].
///
/// This is similar to the [`PointerEvent`] struct, except it also contains the event listener for
/// this event. When you forward an event, this is the data that you can use to build your own
/// custom, [`ForwardedEvent`].
#[derive(Clone, PartialEq, Debug)]
pub struct ListenedEvent<E: IsPointerEvent> {
    /// The pointer involved in this event.
    pub id: PointerId,
    /// The entity that was listening for this event.
    pub listener: Entity,
    /// The entity that this event was originally triggered on.
    pub target: Entity,
    /// The inner event data, if any, for the specific event that was triggered.
    pub inner: E,
}

/// Controls whether the event should bubble up to the entity's parent, or halt.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum Bubble {
    /// Allows this event to bubble up to its parent.
    #[default]
    Up,
    /// Stops this event from bubbling to the next parent.
    Burst,
}
impl Bubble {
    /// Stop this event from bubbling to the next parent.
    pub fn burst(&mut self) {
        *self = Bubble::Burst;
    }
}

/// Used to mark the inner event types for [`PointerEvent`]s.
pub trait IsPointerEvent: Send + Sync + Clone + std::fmt::Debug + Reflect {}

/// Stores the common data needed for all `PointerEvent`s.
#[derive(Clone, PartialEq, Debug)]
pub struct PointerEvent<E: IsPointerEvent> {
    /// The pointer that triggered this event
    pub pointer_id: PointerId,
    /// The location of the pointer during this event
    pub pointer_location: Location,
    /// THe target of this event
    pub target: Entity,
    /// Additional event-specific data. [`Drop`] for example, has an additional field to describe
    /// the `Entity` that is being dropped on the target.
    pub event: E,
}

impl<E: IsPointerEvent> std::fmt::Display for PointerEvent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:?}, {:.1?}, {:?}, {:.1?}",
            self.pointer_id, self.pointer_location.position, self.target, self.event
        ))
    }
}

impl<E: IsPointerEvent + 'static> PointerEvent<E> {
    /// Construct a new `PointerEvent`.
    pub fn new(id: PointerId, location: Location, target: Entity, event: E) -> Self {
        Self {
            pointer_id: id,
            pointer_location: location,
            target: target,
            event,
        }
    }
}

/// Bubbles [`PointerEvent`]s of event type `E`.
///
/// Event bubbling makes it simple for specific entities to listen for specific events. When a
/// `PointerEvent` event is fired, `event_bubbling` will look for an `EventListener` on the event's
/// target entity, then walk up the hierarchy of the entity's ancestors, until a [`Bubble`]`::Pop`
/// is found or the root of the hierarchy is reached.
///
/// For every entity in the hierarchy, this system will look for an [`EventListener`]  matching the
/// event type `E`, and run the `callback` function in the event listener.
///
/// Some `PointerEvent`s cannot be bubbled, and are instead sent to the entire hierarchy.
pub fn event_bubbling<E: IsPointerEvent + 'static>(
    mut commands: Commands,
    mut events: EventReader<PointerEvent<E>>,
    listeners: Query<(Option<&EventListener<E>>, Option<&Parent>)>,
) {
    for event in events.iter() {
        let mut listener = event.target;
        while let Ok((event_listener, parent)) = listeners.get(listener) {
            if let Some(event_listener) = event_listener {
                let event_data = ListenedEvent {
                    id: event.pointer_id,
                    listener,
                    target: event.target,
                    inner: event.event.clone(),
                };
                let mut bubble = Bubble::default();
                let callback = event_listener.callback;
                callback(&mut commands, &event_data, &mut bubble);

                match bubble {
                    Bubble::Up => match parent {
                        Some(parent) => listener = **parent,
                        None => break, // Bubble reached the surface!
                    },
                    Bubble::Burst => break,
                }
            } else {
                match parent {
                    Some(parent) => listener = **parent,
                    None => break, // Bubble reached the surface!
                }
            }
        }
    }
}

/// Fires when a pointer is no longer available.
#[derive(Clone, PartialEq, Debug, Reflect)]
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
impl IsPointerEvent for Over {}

/// Fires when a the pointer crosses out of the bounds of the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Out {
    /// Information about the latest prior picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for Out {}

/// Fires when a pointer button is pressed over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Down {
    /// Pointer button pressed to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for Down {}

/// Fires when a pointer button is released over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Up {
    /// Pointer button lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for Up {}

/// Fires when a pointer sends a pointer down event followed by a pointer up event, with the same
/// `target` entity for both events.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Click {
    /// Pointer button pressed and lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for Click {}

/// Fires while a pointer is moving over the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Move {
    /// Information about the picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for Move {}

/// Fires when the `target` entity receives a pointer down event followed by a pointer move event.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragStart {
    /// Pointer button pressed and moved to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for DragStart {}

/// Fires while the `target` entity is being dragged.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Drag {
    /// Pointer button pressed and moved to trigger this event.
    pub button: PointerButton,
}
impl IsPointerEvent for Drag {}

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragEnd {
    /// Pointer button pressed, moved, and lifted to trigger this event.
    pub button: PointerButton,
}
impl IsPointerEvent for DragEnd {}

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
impl IsPointerEvent for DragEnter {}

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
impl IsPointerEvent for DragOver {}

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
impl IsPointerEvent for DragLeave {}

/// Fires when a pointer drops the `dropped` entity onto the `target` entity.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Drop {
    /// Pointer button lifted to drop.
    pub button: PointerButton,
    /// The entity that was dropped onto the `target` entity.
    pub dropped_entity: Entity,
    /// Information about the picking intersection.
    pub hit: HitData,
}
impl IsPointerEvent for Drop {}

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
    mut pointer_move: EventWriter<PointerEvent<Move>>,
    mut pointer_over: EventWriter<PointerEvent<Over>>,
    mut pointer_out: EventWriter<PointerEvent<Out>>,
    mut pointer_up: EventWriter<PointerEvent<Up>>,
    mut pointer_down: EventWriter<PointerEvent<Down>>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    for move_event in input_moves.iter() {
        for (hovered_entity, hit) in hover_map
            .get(&move_event.pointer_id)
            .iter()
            .flat_map(|h| h.iter().map(|(entity, data)| (*entity, *data)))
        {
            let Some(location) = pointer_location(move_event.pointer_id) else {
                error!("Unable to get location for pointer {:?}", move_event.pointer_id);
                continue;
            };
            pointer_move.send(PointerEvent::new(
                move_event.pointer_id,
                location,
                hovered_entity,
                Move { hit },
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
            .flat_map(|h| h.iter().map(|(entity, data)| (*entity, *data)))
        {
            if let PressDirection::Up = press_event.direction {
                let Some(location) = pointer_location(press_event.pointer_id) else {
                    error!("Unable to get location for pointer {:?}", press_event.pointer_id);
                    continue;
                };
                pointer_up.send(PointerEvent::new(
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
            .flat_map(|h| h.iter().map(|(entity, data)| (*entity, *data)))
        {
            if let PressDirection::Down = press_event.direction {
                let Some(location) = pointer_location(press_event.pointer_id) else {
                    error!("Unable to get location for pointer {:?}", press_event.pointer_id);
                    continue;
                };
                pointer_down.send(PointerEvent::new(
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
        .flat_map(|(id, hashmap)| hashmap.iter().map(|data| (*id, *data.0, *data.1)))
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
            pointer_over.send(PointerEvent::new(
                pointer_id,
                location,
                hovered_entity,
                Over { hit },
            ));
        }
    }

    // If the entity was hovered last frame...
    for (pointer_id, hovered_entity, hit) in previous_hover_map
        .iter()
        .flat_map(|(id, hashmap)| hashmap.iter().map(|data| (*id, *data.0, *data.1)))
    {
        // ...but is now not being hovered...
        if !hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains_key(&hovered_entity))
        {
            let Some(location) = pointer_location(pointer_id) else {
                error!("Unable to get location for pointer {:?}", pointer_id);
                continue;
            };
            pointer_out.send(PointerEvent::new(
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
pub struct DragMap(pub HashMap<(PointerId, PointerButton), Option<Entity>>);

/// Uses pointer events to determine when click and drag events occur.
pub fn send_click_and_drag_events(
    // Input
    mut pointer_down: EventReader<PointerEvent<Down>>,
    mut pointer_up: EventReader<PointerEvent<Up>>,
    mut pointer_move: EventReader<PointerEvent<Move>>,
    mut input_move: EventReader<InputMove>,
    mut input_presses: EventReader<InputPress>,
    pointer_map: Res<PointerMap>,
    pointers: Query<&PointerLocation>,
    // Locals
    mut down_map: Local<HashMap<(PointerId, PointerButton), Option<Entity>>>,
    // Output
    mut drag_map: ResMut<DragMap>,
    mut pointer_click: EventWriter<PointerEvent<Click>>,
    mut pointer_drag_start: EventWriter<PointerEvent<DragStart>>,
    mut pointer_drag_end: EventWriter<PointerEvent<DragEnd>>,
    mut pointer_drag: EventWriter<PointerEvent<Drag>>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    // Only triggers when over an entity
    for PointerEvent {
        pointer_id: pointer,
        pointer_location,
        target,
        event: Move { hit },
    } in pointer_move.iter().cloned()
    {
        for button in PointerButton::iter() {
            let is_pointer_down = matches!(down_map.get(&(pointer, button)), Some(Some(_)));
            let is_pointer_dragging = matches!(drag_map.get(&(pointer, button)), Some(Some(_)));
            if is_pointer_down && !is_pointer_dragging {
                drag_map.insert((pointer, button), Some(target));
                pointer_drag_start.send(PointerEvent::new(
                    pointer,
                    pointer_location.clone(),
                    target,
                    DragStart { button, hit },
                ))
            }
        }
    }

    // Triggers during movement even if not over an entity
    for InputMove {
        pointer_id,
        location,
    } in input_move.iter().cloned()
    {
        for button in PointerButton::iter() {
            let Some(Some(_)) = down_map.get(&(pointer_id, button)) else {
                continue; // To drag, we have to actually be over an entity
            };
            let Some(Some(drag_entity)) = drag_map.get(&(pointer_id, button)) else {
                continue; // To fire a drag event, a drag start event must be made first
            };
            pointer_drag.send(PointerEvent::new(
                pointer_id,
                location.clone(),
                *drag_entity,
                Drag { button },
            ))
        }
    }

    // Triggers when button is released over an entity
    for PointerEvent {
        pointer_id,
        pointer_location,
        target,
        event: Up { button, hit },
    } in pointer_up.iter().cloned()
    {
        let Some(Some(down_entity)) = down_map.insert((pointer_id, button), None) else {
            continue; // Can't have a click without the button being pressed down first
        };
        if down_entity != target {
            continue; // A click starts and ends on the same target
        }
        pointer_click.send(PointerEvent::new(
            pointer_id,
            pointer_location,
            target,
            Click { button, hit },
        ));
    }

    // Triggers when button is pressed over an entity
    for event in pointer_down.iter() {
        let button = event.event.button;
        down_map.insert((event.pointer_id, button), Some(event.target));
    }

    // Triggered for all button presses
    for press in input_presses.iter() {
        if press.direction != pointer::PressDirection::Up {
            continue; // We are only interested in button releases
        }
        let Some(Some(drag_entity)) =
            drag_map.insert((press.pointer_id, press.button), None) else {
                continue;
            };

        let Some(location) = pointer_location(press.pointer_id) else {
                error!("Unable to get location for pointer {:?}", press.pointer_id);
                continue;
            };
        pointer_drag_end.send(PointerEvent::new(
            press.pointer_id,
            location,
            drag_entity,
            DragEnd {
                button: press.button,
            },
        ));
        down_map.insert((press.pointer_id, press.button), None);
    }
}

/// Uses pointer events to determine when drag-over events occur
pub fn send_drag_over_events(
    // Input
    drag_map: Res<DragMap>,
    mut pointer_over: EventReader<PointerEvent<Over>>,
    mut pointer_move: EventReader<PointerEvent<Move>>,
    mut pointer_out: EventReader<PointerEvent<Out>>,
    mut pointer_drag_end: EventReader<PointerEvent<DragEnd>>,
    // Local
    mut drag_over_map: Local<HashMap<(PointerId, PointerButton), HashMap<Entity, HitData>>>,

    // Output
    mut pointer_drag_enter: EventWriter<PointerEvent<DragEnter>>,
    mut pointer_drag_over: EventWriter<PointerEvent<DragOver>>,
    mut pointer_drag_leave: EventWriter<PointerEvent<DragLeave>>,
    mut pointer_drop: EventWriter<PointerEvent<Drop>>,
) {
    // Fire PointerDragEnter events.
    for PointerEvent {
        pointer_id,
        pointer_location,
        target,
        event: Over { hit },
    } in pointer_over.iter().cloned()
    {
        for button in PointerButton::iter() {
            let Some(&Some(dragged)) = drag_map.get(&(pointer_id, button)) else {
                continue; // Get the entity that is being dragged
            };
            if target == dragged {
                continue; // You can't drag an entity over itself
            }
            let drag_entry = drag_over_map.entry((pointer_id, button)).or_default();
            drag_entry.insert(target, hit);
            let event = DragEnter {
                button,
                dragged,
                hit,
            };
            pointer_drag_enter.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                target,
                event,
            ))
        }
    }

    // Fire PointerDragOver events.
    for PointerEvent {
        pointer_id,
        pointer_location,
        target,
        event: Move { hit },
    } in pointer_move.iter().cloned()
    {
        for button in PointerButton::iter() {
            let Some(&Some(dragged)) = drag_map.get(&(pointer_id, button)) else {
                continue; // Get the entity that is being dragged
            };
            if target == dragged {
                continue; // You can't drag an entity over itself
            }
            pointer_drag_over.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                target,
                DragOver {
                    button,
                    dragged,
                    hit,
                },
            ))
        }
    }

    // Fire PointerDragLeave and PointerDrop events when the pointer stops dragging.
    for PointerEvent {
        pointer_id,
        pointer_location,
        target,
        event: DragEnd { button },
    } in pointer_drag_end.iter().cloned()
    {
        let Some(drag_over_set) =
            drag_over_map.get_mut(&(pointer_id, button)) else {
                continue;
            };
        for (dragged_over, hit) in drag_over_set.drain() {
            pointer_drag_leave.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                dragged_over,
                DragLeave {
                    button,
                    dragged: target,
                    hit,
                },
            ));
            pointer_drop.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                dragged_over,
                Drop {
                    button,
                    dropped_entity: target,
                    hit,
                },
            ));
        }
    }

    // Fire PointerDragLeave events when the pointer goes out of the target.
    for PointerEvent {
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
            let Some(&Some(dragged)) = drag_map.get(&(pointer_id, button))  else {
                continue;
            };
            pointer_drag_leave.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                target,
                DragLeave {
                    button,
                    dragged,
                    hit,
                },
            ))
        }
    }
}
