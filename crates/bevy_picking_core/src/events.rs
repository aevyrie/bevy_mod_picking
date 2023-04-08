//! Processes data from input and backends, producing interaction events.

use std::ops::{Deref, DerefMut};

use crate::{
    backend::PickData,
    focus::{HoverMap, PreviousHoverMap},
    pointer::{self, InputMove, InputPress, PointerButton, PointerId, PressDirection},
};
use bevy::{
    ecs::{event::Event, system::EntityCommands},
    prelude::*,
    utils::HashMap,
};

/// Holds a map of entities this pointer is currently interacting with.
#[derive(Debug, Default, Clone, Component)]
pub struct PointerInteraction {
    map: HashMap<Entity, Interaction>,
}
impl Deref for PointerInteraction {
    type Target = HashMap<Entity, Interaction>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl DerefMut for PointerInteraction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// Can be implemented on a custom event to allow [`EventListener`]s to convert [`PointerEvent`]s
/// into the custom event type.
pub trait ForwardedEvent<E: IsPointerEvent>: Event {
    /// Create a new event from [`EventData`].
    fn from_data(event_data: &EventListenerData<E>) -> Self;
}

/// An `EventListener` marks an entity, informing the [`event_bubbling`] system to run the
/// `callback` function when an event of type `E` is being bubbled up the hierarchy and reaches this
/// entity.
#[derive(Component, Clone, Reflect)]
pub struct EventListener<E: IsPointerEvent> {
    #[reflect(ignore)]
    /// A function that is called when the event listener is triggered.
    callback: fn(&mut Commands, &EventListenerData<E>, &mut Bubble),
}

impl<E: IsPointerEvent> EventListener<E> {
    /// Create an [`EventListener`] that will run the supplied `callback` function with access to
    /// bevy [`Commands`] when the pointer event reaches this entity.
    pub fn callback(callback: fn(&mut Commands, &EventListenerData<E>, &mut Bubble)) -> Self {
        Self { callback }
    }

    /// Create an [`EventListener`] that will send an event of type `F` when the listener is
    /// triggered, then continue to bubble the original event up this entity's hierarchy.
    pub fn new_forward_event<F: ForwardedEvent<E>>() -> Self {
        Self {
            callback: |commands: &mut Commands,
                       event_data: &EventListenerData<E>,
                       _bubble: &mut Bubble| {
                let forwarded_event = F::from_data(event_data);
                commands.add(|world: &mut World| {
                    let mut events = world.get_resource_or_insert_with(Events::<F>::default);
                    events.send(forwarded_event);
                });
            },
        }
    }

    /// Create an [`EventListener`] that will send an event of type `F` when the listener is
    /// triggered, then halt bubbling, preventing event listeners of the same type from triggering
    /// on parents of this entity.
    ///
    /// Prefer using `new_forward_event` instead, unless you have a good reason to halt bubbling.
    pub fn new_forward_event_and_halt<F: ForwardedEvent<E>>() -> Self {
        Self {
            callback: |commands: &mut Commands,
                       event_data: &EventListenerData<E>,
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

/// Extends the [`EntityCommands`] trait, allowing you to call these methods when spawning an
/// entity.
pub trait EventListenerCommands {
    /// Listens for events of type `In`; when one bubbles up to this entity, an event of type `Out`
    /// will be sent.
    ///
    /// # Usage
    ///
    /// This will send your custom `MyForwardedEvent`, when this entity receives a `Click`. A
    /// helpful way to read this statement is "forward events of type `Click` to events of type
    /// `MyForwardedEvent`"
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_picking_core::events::*;
    /// # struct MyForwardedEvent;
    /// # impl ForwardedEvent<PointerClick> for MyForwardedEvent {
    /// #    fn from_data(event_data: &EventData<PointerClick>) -> Self {
    /// #         MyForwardedEvent
    /// #     }
    /// # }
    /// # fn my_func(mut commands: Commands){
    /// commands
    ///     .spawn(())
    ///     .forward_events::<Click, MyForwardedEvent>();
    /// # }
    /// ```
    fn forward_events<In: IsPointerEvent, Out: ForwardedEvent<In>>(&mut self) -> &mut Self;

    /// Listens for events of type `In`. When found, an event of type `Out` will be sent. Finally,
    /// bubbling will be halted. See [`event_bubbling`] for details on how bubbling works.
    ///
    /// Prefer using `forward_events` instead, unless you have a good reason to halt bubbling.
    fn forward_events_and_halt<In: IsPointerEvent, Out: ForwardedEvent<In>>(&mut self)
        -> &mut Self;
}

impl<'w, 's, 'a> EventListenerCommands for EntityCommands<'w, 's, 'a> {
    fn forward_events<In: IsPointerEvent, Out: ForwardedEvent<In>>(&mut self) -> &mut Self {
        self.commands().add(|world: &mut World| {
            world.init_resource::<Events<Out>>();
        });
        self.insert(EventListener::<In>::new_forward_event::<Out>());
        self
    }
    fn forward_events_and_halt<In: IsPointerEvent, Out: ForwardedEvent<In>>(
        &mut self,
    ) -> &mut Self {
        self.commands().add(|world: &mut World| {
            world.init_resource::<Events<Out>>();
        });
        self.insert(EventListener::<In>::new_forward_event_and_halt::<Out>());
        self
    }
}

/// Data from a pointer event, for use with [`EventListener`]s and event forwarding.
///
/// This is similar to the [`PointerEvent`] struct, except it also contains the event listener for
/// this event, as well as the ability to stop bubbling this event. When you forward an event, this
/// is the data that you can use to build your own custom, [`ForwardedEvent`].
#[derive(Clone, PartialEq, Debug)]
pub struct EventListenerData<E: IsPointerEvent> {
    /// The pointer involved in this event.
    id: PointerId,
    /// The entity that was listening for this event.
    listener: Entity,
    /// The entity that this event was originally triggered on.
    target: Entity,
    /// The inner event data, if any, for the specific event that was triggered.
    event: E,
}
impl<E: IsPointerEvent> EventListenerData<E> {
    /// Get the [`PointerId`] associated with this event.
    pub fn id(&self) -> PointerId {
        self.id
    }

    /// Get the entity that was listening for this event. Note this is ***not*** the target entity
    /// of the event - though it can be - it is an ancestor of the target entity that has an
    /// [`EventListener`] which was triggered.
    pub fn listener(&self) -> Entity {
        self.listener
    }

    /// Get the target entity of the event. E.g. the entity that was clicked on.
    pub fn target(&self) -> Entity {
        self.target
    }

    /// The inner event that may contain event-specific data.
    pub fn event(&self) -> &E {
        &self.event
    }
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
    pointer_id: PointerId,
    target: Entity,
    event: E,
}
impl<E: IsPointerEvent + 'static> PointerEvent<E> {
    /// Construct a new `PointerEvent`.
    pub fn new(id: &PointerId, target: &Entity, event: E) -> Self {
        Self {
            pointer_id: *id,
            target: *target,
            event,
        }
    }

    /// Returns the [`PointerId`] of this event.
    pub fn pointer_id(&self) -> PointerId {
        self.pointer_id
    }

    /// Returns the target [`Entity`] of this event.
    pub fn target(&self) -> Entity {
        self.target
    }

    /// Returns internal event data. [`Drop`] for example, has an additional field to describe the
    /// `Entity` being dropped on the target.
    pub fn event_data(&self) -> E {
        self.event.to_owned()
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
                let event_data = EventListenerData {
                    id: event.pointer_id,
                    listener,
                    target: event.target,
                    event: event.event.clone(),
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
impl<E: IsPointerEvent> std::fmt::Display for PointerEvent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Target: \x1b[0;1;0m{:?}\x1b[0m, ID: \x1b[0;1;0m{:?}\x1b[0m",
            self.target, self.pointer_id
        )
    }
}

/// Fires when a pointer is no longer available.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct PointerCancel {
    /// ID of the pointer that was cancelled.
    #[reflect(ignore)]
    pub pointer_id: PointerId,
}

/// Fires when a the pointer crosses into the bounds of the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Over {
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for Over {}

/// Fires when a the pointer crosses out of the bounds of the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Out;
impl IsPointerEvent for Out {}

/// Fires when a pointer button is pressed over the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Down {
    /// Pointer button pressed to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for Down {}

/// Fires when a pointer button is released over the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Up {
    /// Pointer button lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for Up {}

/// Fires when a pointer sends a pointer down event followed by a pointer up event, with the same
/// `target` entity for both events.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Click {
    /// Pointer button pressed and lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for Click {}

/// Fires while a pointer is moving over the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Move {
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for Move {}

/// Fires when the `target` entity receives a pointer down event followed by a pointer move event.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct DragStart {
    /// Pointer button pressed and moved to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for DragStart {}

/// Fires while the `target` entity is being dragged.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Drag {
    /// Pointer button pressed and moved to trigger this event.
    pub button: PointerButton,
}
impl IsPointerEvent for Drag {}

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct DragEnd {
    /// Pointer button pressed, moved, and lifted to trigger this event.
    pub button: PointerButton,
}
impl IsPointerEvent for DragEnd {}

/// Fires when a pointer dragging the `dragged` entity enters the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct DragEnter {
    /// Pointer button pressed to enter drag.
    pub button: PointerButton,
    /// The entity that was being dragged when the pointer entered the `target` entity.
    pub dragged: Entity,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for DragEnter {}

/// Fires while the `dragged` entity is being dragged over the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct DragOver {
    /// Pointer button pressed while dragging over.
    pub button: PointerButton,
    /// The entity that was being dragged when the pointer was over the `target` entity.
    pub dragged: Entity,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for DragOver {}

/// Fires when a pointer dragging the `dragged` entity leaves the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct DragLeave {
    /// Pointer button pressed while leaving drag.
    pub button: PointerButton,
    /// The entity that was being dragged when the pointer left the `target` entity.
    pub dragged: Entity,
}
impl IsPointerEvent for DragLeave {}

/// Fires when a pointer drops the `dropped` entity onto the `target` entity.
#[derive(Copy, Clone, PartialEq, Debug, Reflect)]
pub struct Drop {
    /// Pointer button lifted to drop.
    pub button: PointerButton,
    /// The entity that was dropped onto the `target` entity.
    pub dropped_entity: Entity,
    /// Information about the picking intersection.
    pub pick_data: PickData,
}
impl IsPointerEvent for Drop {}

/// Generates pointer events from input data
pub fn pointer_events(
    // Input
    mut input_presses: EventReader<InputPress>,
    mut input_moves: EventReader<pointer::InputMove>,
    hover_map: Res<HoverMap>,
    previous_hover_map: Res<PreviousHoverMap>,
    // Output
    mut pointer_move: EventWriter<PointerEvent<Move>>,
    mut pointer_over: EventWriter<PointerEvent<Over>>,
    mut pointer_out: EventWriter<PointerEvent<Out>>,
    mut pointer_up: EventWriter<PointerEvent<Up>>,
    mut pointer_down: EventWriter<PointerEvent<Down>>,
) {
    for move_event in input_moves.iter() {
        for (hovered_entity, pick_data) in hover_map
            .get(&move_event.pointer_id())
            .iter()
            .flat_map(|h| h.iter())
        {
            pointer_move.send(PointerEvent::new(
                &move_event.pointer_id(),
                hovered_entity,
                Move {
                    pick_data: pick_data.clone(),
                },
            ))
        }
    }

    for press_event in input_presses.iter() {
        // We use the previous hover map because we want to consider pointers that just left the
        // entity. Without this, touch inputs would never send up events because they are lifted up
        // and leave the bounds of the entity at the same time.
        for (hovered_entity, pick_data) in previous_hover_map
            .get(&press_event.pointer_id())
            .iter()
            .flat_map(|h| h.iter())
        {
            if let PressDirection::Up = press_event.direction() {
                let pointer_id = &press_event.pointer_id();
                pointer_up.send(PointerEvent::new(
                    pointer_id,
                    hovered_entity,
                    Up {
                        button: press_event.button(),
                        pick_data: pick_data.clone(),
                    },
                ))
            }
        }
        for (hovered_entity, pick_data) in hover_map
            .get(&press_event.pointer_id())
            .iter()
            .flat_map(|h| h.iter())
        {
            if let PressDirection::Down = press_event.direction() {
                pointer_down.send(PointerEvent::new(
                    &press_event.pointer_id(),
                    hovered_entity,
                    Down {
                        button: press_event.button(),
                        pick_data: pick_data.clone(),
                    },
                ))
            }
        }
    }

    // If the entity is hovered...
    for (pointer_id, (hovered_entity, pick_data)) in hover_map
        .iter()
        .flat_map(|(p, h)| h.iter().map(|h| (p.to_owned(), h)))
    {
        // ...but was not hovered last frame...
        if !previous_hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains_key(hovered_entity))
        {
            pointer_over.send(PointerEvent::new(
                &pointer_id,
                hovered_entity,
                Over {
                    pick_data: pick_data.clone(),
                },
            ));
        }
    }

    // If the entity was hovered last frame...
    for (pointer_id, (hovered_entity, _)) in previous_hover_map
        .iter()
        .flat_map(|(p, h)| h.iter().map(|h| (p.to_owned(), h)))
    {
        // ...but is now not being hovered...
        if !hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains_key(hovered_entity))
        {
            pointer_out.send(PointerEvent::new(&pointer_id, hovered_entity, Out));
        }
    }
}

/// Uses pointer events to update [`PointerInteraction`] and [`Interaction`] components.
pub fn interactions_from_events(
    // Input
    mut pointer_over: EventReader<PointerEvent<Over>>,
    mut pointer_out: EventReader<PointerEvent<Out>>,
    mut pointer_up: EventReader<PointerEvent<Up>>,
    mut pointer_down: EventReader<PointerEvent<Down>>,
    // Outputs
    mut pointers: Query<(&PointerId, &mut PointerInteraction)>,
    mut interact: Query<&mut Interaction>,
) {
    for event in pointer_over.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut interact);
    }
    for event in pointer_down.iter() {
        update_interactions(event, Interaction::Clicked, &mut pointers, &mut interact);
    }
    for event in pointer_up.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut interact);
    }
    for event in pointer_out.iter() {
        update_interactions(event, Interaction::None, &mut pointers, &mut interact);
    }
}

fn update_interactions<E: IsPointerEvent>(
    event: &PointerEvent<E>,
    new_interaction: Interaction,
    pointer_interactions: &mut Query<(&PointerId, &mut PointerInteraction)>,
    entity_interactions: &mut Query<&mut Interaction>,
) {
    if let Some(mut interaction_map) = pointer_interactions
        .iter_mut()
        .find_map(|(id, interaction)| (*id == event.pointer_id).then_some(interaction))
    {
        interaction_map.insert(event.target, new_interaction);
        if let Ok(mut interaction) = entity_interactions.get_mut(event.target) {
            *interaction = new_interaction;
        }
        interaction_map
            .retain(|_, i| i != &Interaction::None || new_interaction != Interaction::None);
    };
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
    // Locals
    mut down_map: Local<HashMap<(PointerId, PointerButton), Option<Entity>>>,
    // Output
    mut drag_map: ResMut<DragMap>,
    mut pointer_click: EventWriter<PointerEvent<Click>>,
    mut pointer_drag_start: EventWriter<PointerEvent<DragStart>>,
    mut pointer_drag_end: EventWriter<PointerEvent<DragEnd>>,
    mut pointer_drag: EventWriter<PointerEvent<Drag>>,
) {
    // Only triggers when over an entity
    for move_event in pointer_move.iter() {
        for button in PointerButton::iter() {
            let moving_pointer_is_down = matches!(
                down_map.get(&(move_event.pointer_id(), button)),
                Some(Some(_))
            );
            let pointer_not_in_drag_map = matches!(
                drag_map.get(&(move_event.pointer_id(), button)),
                Some(None) | None
            );
            if moving_pointer_is_down && pointer_not_in_drag_map {
                drag_map.insert((move_event.pointer_id(), button), Some(move_event.target()));
                pointer_drag_start.send(PointerEvent::new(
                    &move_event.pointer_id(),
                    &move_event.target(),
                    DragStart {
                        button,
                        pick_data: move_event.event.pick_data.clone(),
                    },
                ))
            }
        }
    }

    // Triggers during movement even if not over an entity
    for move_event in input_move.iter() {
        for button in PointerButton::iter() {
            let Some(Some(_)) = down_map.get(&(move_event.pointer_id(), button)) else {
                continue; // To drag, we have to actually be over an entity
            };
            let Some(Some(drag_entity)) = drag_map.get(&(move_event.pointer_id(), button)) else {
                continue; // To fire a drag event, a drag start event must be made first
            };
            pointer_drag.send(PointerEvent::new(
                &move_event.pointer_id(),
                drag_entity,
                Drag { button },
            ))
        }
    }

    // Triggers when button is released over an entity
    for up_event in pointer_up.iter() {
        let button = up_event.event.button;
        let Some(Some(down_entity)) = down_map.insert((up_event.pointer_id(), button), None) else {
            continue; // Can't have a click without the button being pressed down first
        };
        if down_entity != up_event.target() {
            continue; // A click starts and ends on the same target
        }
        pointer_click.send(PointerEvent::new(
            &up_event.pointer_id(),
            &up_event.target(),
            Click {
                button,
                pick_data: up_event.event.pick_data.clone(),
            },
        ));
    }

    // Triggers when button is pressed over an entity
    for event in pointer_down.iter() {
        let button = event.event.button;
        down_map.insert((event.pointer_id(), button), Some(event.target()));
    }

    // Triggered for all button presses
    for press in input_presses.iter() {
        if press.direction() != pointer::PressDirection::Up {
            continue; // We are only interested in button releases
        }
        let Some(Some(drag_entity)) =
            drag_map.insert((press.pointer_id(), press.button()), None) else {
                continue;
            };
        pointer_drag_end.send(PointerEvent::new(
            &press.pointer_id(),
            &drag_entity,
            DragEnd {
                button: press.button(),
            },
        ));
        down_map.insert((press.pointer_id(), press.button()), None);
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
    mut drag_over_map: Local<HashMap<(PointerId, PointerButton), HashMap<Entity, PickData>>>,

    // Output
    mut pointer_drag_enter: EventWriter<PointerEvent<DragEnter>>,
    mut pointer_drag_over: EventWriter<PointerEvent<DragOver>>,
    mut pointer_drag_leave: EventWriter<PointerEvent<DragLeave>>,
    mut pointer_drop: EventWriter<PointerEvent<Drop>>,
) {
    // Fire PointerDragEnter events.
    for over_event in pointer_over.iter() {
        for button in PointerButton::iter() {
            let Some(&Some(dragged)) = drag_map.get(&(over_event.pointer_id(), button)) else {
                continue;
            };
            if over_event.target() == dragged {
                continue; // You can't drag an entity over itself
            }
            let drag_entry = drag_over_map
                .entry((over_event.pointer_id(), button))
                .or_default();
            drag_entry.insert(over_event.target(), over_event.event.pick_data);
            pointer_drag_enter.send(PointerEvent::new(
                &over_event.pointer_id(),
                &over_event.target(),
                DragEnter {
                    button,
                    dragged,
                    pick_data: over_event.event.pick_data.clone(),
                },
            ))
        }
    }

    // Fire PointerDragOver events.
    for move_event in pointer_move.iter() {
        for button in PointerButton::iter() {
            let Some(&Some(dragged)) = drag_map.get(&(move_event.pointer_id(), button)) else {
                continue;
            };
            if move_event.target() == dragged {
                continue; // You can't drag an entity over itself
            }
            pointer_drag_over.send(PointerEvent::new(
                &move_event.pointer_id(),
                &move_event.target(),
                DragOver {
                    button,
                    dragged,
                    pick_data: move_event.event.pick_data.clone(),
                },
            ))
        }
    }

    // Fire PointerDragLeave and PointerDrop events when the pointer stops dragging.
    for drag_end_event in pointer_drag_end.iter() {
        let button = drag_end_event.event.button;
        let Some(drag_over_set) =
            drag_over_map.get_mut(&(drag_end_event.pointer_id(), button)) else {
                continue;
            };
        for (dragged_over, pick_data) in drag_over_set.drain() {
            pointer_drag_leave.send(PointerEvent::new(
                &drag_end_event.pointer_id(),
                &dragged_over,
                DragLeave {
                    button,
                    dragged: drag_end_event.target(),
                },
            ));
            pointer_drop.send(PointerEvent::new(
                &drag_end_event.pointer_id(),
                &dragged_over,
                Drop {
                    button,
                    dropped_entity: drag_end_event.target(),
                    pick_data,
                },
            ));
        }
    }

    // Fire PointerDragLeave events when the pointer goes out of the target.
    for out_event in pointer_out.iter() {
        let out_pointer = out_event.pointer_id();
        let out_target = &out_event.target();
        for button in PointerButton::iter() {
            let Some(dragged_over) = drag_over_map.get_mut(&(out_pointer, button)) else {
                continue;
            };
            if dragged_over.remove(out_target).is_none() {
                continue;
            }
            let Some(&Some(dragged)) = drag_map.get(&(out_pointer, button))  else {
                continue;
            };
            pointer_drag_leave.send(PointerEvent::new(
                &out_pointer,
                out_target,
                DragLeave { button, dragged },
            ))
        }
    }
}
