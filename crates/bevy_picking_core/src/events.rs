//! Processes data from input and backends, producing interaction events.

use std::marker::PhantomData;

use crate::{
    backend::HitData,
    focus::{HoverMap, PreviousHoverMap},
    pointer::{
        self, InputMove, InputPress, Location, PointerButton, PointerId, PointerLocation,
        PointerMap, PressDirection,
    },
    PickSet,
};
use bevy::{ecs::system::Command, prelude::*, utils::HashMap};

/// Adds event listening and bubbling support for event `E`.
pub struct EventListenerPlugin<E>(PhantomData<E>);

impl<E> Default for EventListenerPlugin<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: IsPointerEvent> Plugin for EventListenerPlugin<E> {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventCallbackGraph::<E>::default())
            .add_systems(
                (
                    EventCallbackGraph::<E>::build.run_if(on_event::<PointerEvent<E>>()),
                    event_bubbling::<E>.run_if(on_event::<PointerEvent<E>>()),
                )
                    .chain()
                    .in_set(PickSet::EventListeners),
            );
    }
}

enum CallbackSystem<E: IsPointerEvent> {
    Empty,
    New(Box<dyn System<In = ListenedEvent<E>, Out = Bubble>>),
    Initialized(Box<dyn System<In = ListenedEvent<E>, Out = Bubble>>),
}

impl<E: IsPointerEvent> CallbackSystem<E> {
    fn is_initialized(&self) -> bool {
        matches!(self, CallbackSystem::Initialized(_))
    }

    fn run(&mut self, world: &mut World, event_data: ListenedEvent<E>) -> Bubble {
        if !self.is_initialized() {
            let mut temp = CallbackSystem::Empty;
            std::mem::swap(self, &mut temp);
            if let CallbackSystem::New(mut system) = temp {
                system.initialize(world);
                *self = CallbackSystem::Initialized(system);
            }
        }
        match self {
            CallbackSystem::Initialized(system) => {
                let result = system.run(event_data, world);
                system.apply_buffers(world);
                result
            }
            _ => unreachable!(),
        }
    }
}

/// An `OnPointer` component marks an entity, informing the [`event_bubbling`] system to run the
/// `callback` function when an event of type `E` is being bubbled up the hierarchy and reaches this
/// entity.
#[derive(Component, Reflect)]
pub struct OnPointer<E: IsPointerEvent> {
    #[reflect(ignore)]
    /// A function that is called when the event listener is triggered.
    callback: CallbackSystem<E>,
}

impl<E: IsPointerEvent> OnPointer<E> {
    /// Run a callback system when this event listener is triggered.
    pub fn run_callback<M>(callback: impl IntoSystem<ListenedEvent<E>, Bubble, M>) -> Self {
        Self {
            callback: CallbackSystem::New(Box::new(IntoSystem::into_system(callback))),
        }
    }

    /// Add a command to the [`CommandQueue`](bevy::ecs::system::CommandQueue) when when this event
    /// listener is triggered.
    pub fn add_command<C: From<ListenedEvent<E>> + Command + Send + Sync + 'static>() -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut commands: Commands| {
                commands.add(C::from(event));
                Bubble::Up
            },
        )
    }

    /// Send an event `F` when this event listener is triggered. `F` must implement
    /// `From<ListenedEvent<E>>`.
    pub fn send_event<F: Event + From<ListenedEvent<E>>>() -> Self {
        Self::run_callback(|In(event): In<ListenedEvent<E>>, mut ev: EventWriter<F>| {
            ev.send(F::from(event));
            Bubble::Up
        })
    }

    /// Take the boxed system callback out of this listener, leaving an empty one behind.
    fn take(&mut self) -> CallbackSystem<E> {
        let mut temp = CallbackSystem::Empty;
        std::mem::swap(&mut self.callback, &mut temp);
        temp
    }
}

/// Data from a pointer event returned by an [`OnPointer`].
///
/// This is similar to the [`PointerEvent`] struct, with the addition of event listener data.
#[derive(Clone, PartialEq, Debug)]
pub struct ListenedEvent<E: IsPointerEvent> {
    /// The pointer involved in this event.
    pub pointer_id: PointerId,
    /// The location of the pointer during this event
    pub pointer_location: Location,
    /// The entity that was listening for this event.
    pub listener: Entity,
    /// The entity that this event was originally triggered on.
    pub target: Entity,
    /// Event-specific information, e.g. data specific to `Drag` events, that isn't shared amongst
    /// all pointer events. This contains the [`HitData`] if it is available.
    pub pointer_event: E,
}

impl<E: IsPointerEvent> std::ops::Deref for ListenedEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.pointer_event
    }
}

/// Determines whether an event should continue to bubble up the entity hierarchy.
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
    /// The target of this event
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

impl<E: IsPointerEvent> std::ops::Deref for PointerEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<E: IsPointerEvent + 'static> PointerEvent<E> {
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

/// In order to traverse the entity hierarchy and read events, we need to extract the callbacks out
/// of their components before they can be run. This is because running callbacks requires mutable
/// access to the [`World`], which we can't do if we are also trying to mutate the [`OnPointer`]'s
/// inner callback state.
#[derive(Resource)]
pub struct EventCallbackGraph<E: IsPointerEvent> {
    /// All the events of type `E` that were emitted this frame, and encountered an [`OnPointer<E>`]
    /// while traversing the entity hierarchy. The `Entity` in the tuple is the root node to use
    /// when traversing the listener graph.
    events: Vec<(PointerEvent<E>, Entity)>,
    /// Traversing the entity hierarchy for each event can visit the same entity multiple times.
    /// Storing the callbacks for each of these potentially visited entities in a graph structure is
    /// necessary for a few reasons:
    ///
    /// - Callback systems cannot implement `Clone`, so we can only have one copy of each callback
    ///   system.
    /// - For complex hierarchies, this is more memory efficient.
    /// - This allows us to jump to the next listener in the hierarchy without unnecessary
    ///   traversal. When bubbling many events of the same type `E` through the same entity tree,
    ///   this can save a significant amount of work.
    listener_graph: HashMap<Entity, (CallbackSystem<E>, Option<Entity>)>,
}

impl<E: IsPointerEvent> EventCallbackGraph<E> {
    fn build(
        mut events: EventReader<PointerEvent<E>>,
        mut listeners: Query<(Option<&mut OnPointer<E>>, Option<&Parent>)>,
        mut callbacks: ResMut<EventCallbackGraph<E>>,
    ) {
        let mut filtered_events = Vec::new();
        let mut listener_map = HashMap::new();

        for event in events.iter() {
            let mut this_node = event.target;
            let mut prev_node = this_node;
            let mut root_node = None;

            'bubble_traversal: loop {
                if let Some((_, next_node)) = listener_map.get(&this_node) {
                    if root_node.is_none() {
                        root_node = Some(this_node);
                    }
                    // If the current entity is already in the map, use it to jump ahead
                    match next_node {
                        Some(next_node) => this_node = *next_node,
                        None => break 'bubble_traversal, // Bubble reached the surface!
                    }
                } else if let Ok((event_listener, parent)) = listeners.get_mut(this_node) {
                    // Otherwise, get the current entity's data with a query
                    if let Some(mut event_listener) = event_listener {
                        // If it has an event listener, we need to add it to the map
                        listener_map.insert(this_node, (event_listener.take(), None));
                        if let Some((_, prev_nodes_next_node)) = listener_map.get_mut(&prev_node) {
                            if prev_node != this_node {
                                *prev_nodes_next_node = Some(this_node);
                            }
                        }
                        if root_node.is_none() {
                            root_node = Some(this_node);
                        }
                        prev_node = this_node;
                    }
                    match parent {
                        Some(parent) => this_node = **parent,
                        None => break 'bubble_traversal, // Bubble reached the surface!
                    }
                } else {
                    // This can be reached if the entity targeted by the event was deleted before
                    // the bubbling system could run.
                    break 'bubble_traversal;
                }
            }
            if let Some(root_node) = root_node {
                // Only add events if they interact with an event listener.
                filtered_events.push((event.to_owned(), root_node));
            }
        }
        callbacks.listener_graph = listener_map;
        callbacks.events = filtered_events;
    }
}

impl<E: IsPointerEvent> Default for EventCallbackGraph<E> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            listener_graph: HashMap::new(),
        }
    }
}

/// Bubbles [`PointerEvent`]s of event type `E`.
///
/// Event bubbling makes it simple for specific entities to listen for specific events. When a
/// `PointerEvent` event is fired, `event_bubbling` will look for an `OnPointer` on the event's
/// target entity, then walk up the hierarchy of the entity's ancestors, until a [`Bubble`]`::Pop`
/// is found or the root of the hierarchy is reached.
///
/// For every entity in the hierarchy, this system will look for an [`OnPointer`] matching the event
/// type `E`, and run the `callback` function in the event listener.
pub fn event_bubbling<E: IsPointerEvent + 'static>(world: &mut World) {
    let Some(mut callbacks) = world.remove_resource::<EventCallbackGraph<E>>() else {
        return
    };
    world.insert_resource(EventCallbackGraph::<E>::default());

    for (event, root_node) in callbacks.events.iter() {
        let mut this_node = *root_node;
        'bubble_traversal: while let Some((callback, next_node)) =
            callbacks.listener_graph.get_mut(&this_node)
        {
            let event_data = ListenedEvent {
                pointer_id: event.pointer_id,
                pointer_location: event.pointer_location.clone(),
                listener: this_node,
                target: event.target,
                pointer_event: event.event.clone(),
            };
            let callback_result = callback.run(world, event_data);
            if callback_result == Bubble::Burst {
                break 'bubble_traversal;
            }
            match next_node {
                Some(next_node) => this_node = *next_node,
                _ => break 'bubble_traversal,
            }
        }
    }

    let mut listeners = world.query::<&mut OnPointer<E>>();

    for (entity, (callback, _)) in callbacks.listener_graph.drain() {
        if let Ok(mut listener) = listeners.get_mut(world, entity) {
            listener.callback = callback;
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
    /// The total distance vector of a drag, measured from drag start to the current position.
    pub distance: Vec2,
    /// The change in position since the last drag event.
    pub delta: Vec2,
}
impl IsPointerEvent for Drag {}

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct DragEnd {
    /// Pointer button pressed, moved, and lifted to trigger this event.
    pub button: PointerButton,
    /// The vector of drag movement measured from start to final pointer position.
    pub distance: Vec2,
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
pub struct DragMap(pub HashMap<(PointerId, PointerButton), Option<DragEntry>>);

/// An entry in the [`DragMap`].
#[derive(Debug)]
pub struct DragEntry {
    /// The entity being dragged.
    pub target: Entity,
    /// The position of the pointer at drag start.
    pub start_pos: Vec2,
    /// The latest position of the pointer during this drag, used to compute deltas.
    pub latest_pos: Vec2,
}

/// Uses pointer events to determine when click and drag events occur.
pub fn send_click_and_drag_events(
    // Input
    mut pointer_down: EventReader<PointerEvent<Down>>,
    mut pointer_up: EventReader<PointerEvent<Up>>,
    mut input_move: EventReader<InputMove>,
    mut input_presses: EventReader<InputPress>,
    pointer_map: Res<PointerMap>,
    pointers: Query<&PointerLocation>,
    // Locals
    mut down_map: Local<HashMap<(PointerId, PointerButton), Option<PointerEvent<Down>>>>,
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

    // Triggers during movement even if not over an entity
    for InputMove {
        pointer_id,
        location,
    } in input_move.iter().cloned()
    {
        for button in PointerButton::iter() {
            let Some(Some(down)) = down_map.get(&(pointer_id, button)) else {
                continue;
            };

            if !matches!(drag_map.get(&(pointer_id, button)), Some(Some(_))) {
                drag_map.insert(
                    (pointer_id, button),
                    Some(DragEntry {
                        target: down.target,
                        start_pos: down.pointer_location.position,
                        latest_pos: down.pointer_location.position,
                    }),
                );
                pointer_drag_start.send(PointerEvent::new(
                    pointer_id,
                    down.pointer_location.clone(),
                    down.target,
                    DragStart {
                        button,
                        hit: down.hit,
                    },
                ))
            }

            if let Some(Some(drag)) = drag_map.get_mut(&(pointer_id, button)) {
                let drag_event = Drag {
                    button,
                    distance: location.position - drag.start_pos,
                    delta: location.position - drag.latest_pos,
                };
                drag.latest_pos = location.position;
                pointer_drag.send(PointerEvent::new(
                    pointer_id,
                    location.clone(),
                    drag.target,
                    drag_event,
                ))
            }
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
        let Some(Some(down)) = down_map.insert((pointer_id, button), None) else {
            continue; // Can't have a click without the button being pressed down first
        };
        if down.target != target {
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
        let button = event.button;
        down_map.insert((event.pointer_id, button), Some(event.clone()));
    }

    // Triggered for all button presses
    for press in input_presses.iter() {
        if press.direction != pointer::PressDirection::Up {
            continue; // We are only interested in button releases
        }
        let Some(Some(drag)) =
            drag_map.insert((press.pointer_id, press.button), None) else {
                continue;
            };

        let Some(location) = pointer_location(press.pointer_id) else {
                error!("Unable to get location for pointer {:?}", press.pointer_id);
                continue;
            };
        let drag_end = DragEnd {
            button: press.button,
            distance: drag.latest_pos - drag.start_pos,
        };
        pointer_drag_end.send(PointerEvent::new(
            press.pointer_id,
            location,
            drag.target,
            drag_end,
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
            let Some(Some(drag)) = drag_map.get(&(pointer_id, button)) else {
                continue; // Get the entity that is being dragged
            };
            if target == drag.target {
                continue; // You can't drag an entity over itself
            }
            let drag_entry = drag_over_map.entry((pointer_id, button)).or_default();
            drag_entry.insert(target, hit);
            let event = DragEnter {
                button,
                dragged: drag.target,
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
            let Some(Some(drag)) = drag_map.get(&(pointer_id, button)) else {
                continue; // Get the entity that is being dragged
            };
            if target == drag.target {
                continue; // You can't drag an entity over itself
            }
            pointer_drag_over.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                target,
                DragOver {
                    button,
                    dragged: drag.target,
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
        event: DragEnd {
            button,
            distance: _,
        },
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
            let Some(Some(drag)) = drag_map.get(&(pointer_id, button))  else {
                continue;
            };
            pointer_drag_leave.send(PointerEvent::new(
                pointer_id,
                pointer_location.clone(),
                target,
                DragLeave {
                    button,
                    dragged: drag.target,
                    hit,
                },
            ))
        }
    }
}
