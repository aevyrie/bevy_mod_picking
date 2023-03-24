//! Processes data from input and backends, producing interaction events.

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{
    focus::{HoverMap, PreviousHoverMap},
    pointer::{self, InputMove, InputPress, PointerButton, PointerId, PressDirection},
};
use bevy::{
    ecs::{event::Event, system::EntityCommands},
    prelude::*,
    utils::{HashMap, HashSet},
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
    fn from_data(event_data: &EventData<E>) -> Self;
}

/// An `EventListener` marks an entity, informing the [`event_bubbling`] system to run the
/// `callback` function when an event of type `E` is being bubbled up the hierarchy and reaches this
/// entity.
#[derive(Component, Clone, Reflect)]
pub struct EventListener<E: IsPointerEvent> {
    #[reflect(ignore)]
    /// A function that is called when the event listener is triggered.
    callback: fn(&mut Commands, &EventData<E>, &mut Bubble),
}

impl<E: IsPointerEvent> EventListener<E> {
    /// Create an [`EventListener`] that will run the supplied `callback` function with access to
    /// bevy [`Commands`] when the pointer event reaches this entity.
    pub fn callback(callback: fn(&mut Commands, &EventData<E>, &mut Bubble)) -> Self {
        Self { callback }
    }

    /// Create an [`EventListener`] that will send an event of type `F` when the listener is
    /// triggered, then continue to bubble the original event up this entity's hierarchy.
    pub fn new_forward_event<F: ForwardedEvent<E>>() -> Self {
        Self {
            callback: |commands: &mut Commands, event_data: &EventData<E>, _bubble: &mut Bubble| {
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
            callback: |commands: &mut Commands, event_data: &EventData<E>, bubble: &mut Bubble| {
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
    /// This will send your custom `MyForwardedEvent`, when this entity receives a `PointerClick`. A
    /// helpful way to read this statement is "forward events of type `PointerClick` to events of
    /// type `MyForwardedEvent`"
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_picking_core::output::*;
    /// # struct MyForwardedEvent;
    /// # impl ForwardedEvent<PointerClick> for MyForwardedEvent {
    /// #    fn from_data(event_data: &EventData<PointerClick>) -> Self {
    /// #         MyForwardedEvent
    /// #     }
    /// # }
    /// # fn my_func(mut commands: Commands){
    /// commands
    ///     .spawn(())
    ///     .forward_events::<PointerClick, MyForwardedEvent>();
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
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EventData<E: IsPointerEvent> {
    /// The pointer involved in this event.
    id: PointerId,
    /// The entity that was listening for this event.
    listener: Entity,
    /// The entity that this event was originally triggered on.
    target: Entity,
    /// The inner event data, if any, for the specific event that was triggered.
    event: E::InnerEventType,
}
impl<E: IsPointerEvent> EventData<E> {
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
    pub fn event(&self) -> &E::InnerEventType {
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

/// This trait restricts the types of events that can be used as [`PointerEvent`]s.
pub trait IsPointerEvent: Send + Sync + Display + Clone + 'static {
    /// The inner event type of the [`PointerEvent`].
    type InnerEventType: Send + Sync + Clone + std::fmt::Debug;
    /// Create a new `PointerEvent`.
    fn new(id: &PointerId, target: &Entity, event: Self::InnerEventType) -> Self;
    /// Get the [`PointerId`] of this event.
    fn pointer_id(&self) -> PointerId;
    /// Get the target entity of this event.
    fn target(&self) -> Entity;
    /// Get the inner event entity of this event.
    fn event_data(&self) -> Self::InnerEventType;
}

/// Stores the common data needed for all `PointerEvent`s.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PointerEvent<E: Send + Sync + Clone + Reflect> {
    pointer_id: PointerId,
    target: Entity,
    event: E,
}
impl<E: Clone + Send + Sync + std::fmt::Debug + Reflect + 'static> IsPointerEvent
    for PointerEvent<E>
{
    type InnerEventType = E;

    fn new(id: &PointerId, target: &Entity, event: E) -> Self {
        Self {
            pointer_id: *id,
            target: *target,
            event,
        }
    }

    fn pointer_id(&self) -> PointerId {
        self.pointer_id
    }

    fn target(&self) -> Entity {
        self.target
    }

    fn event_data(&self) -> Self::InnerEventType {
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
pub fn event_bubbling<E: Clone + Send + Sync + std::fmt::Debug + Reflect + 'static>(
    mut commands: Commands,
    mut events: EventReader<PointerEvent<E>>,
    listeners: Query<(Option<&EventListener<PointerEvent<E>>>, Option<&Parent>)>,
) {
    for event in events.iter() {
        let mut listener = event.target;
        while let Ok((event_listener, parent)) = listeners.get(listener) {
            if let Some(event_listener) = event_listener {
                let event_data = EventData {
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
impl<E: Clone + Send + Sync + Reflect> std::fmt::Display for PointerEvent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Target: \x1b[0;1;0m{:?}\x1b[0m, ID: \x1b[0;1;0m{:?}\x1b[0m",
            self.target, self.pointer_id
        )
    }
}

/// Fires when a pointer is no longer available.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct PointerCancel {
    /// ID of the pointer that was cancelled.
    #[reflect(ignore)]
    pub pointer_id: PointerId,
}

/// Fires when a the pointer crosses into the bounds of the `target` entity.
pub type PointerOver = PointerEvent<Over>;
/// The inner [`PointerEvent`] type for [`PointerOver`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Over;

/// Fires when a the pointer crosses out of the bounds of the `target` entity.
pub type PointerOut = PointerEvent<Out>;
/// The inner [`PointerEvent`] type for [`PointerOut`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Out;

/// Fires when a pointer button is pressed over the `target` entity.
pub type PointerDown = PointerEvent<Down>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
/// The inner [`PointerEvent`] type for [`PointerDown`].
pub struct Down {
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires when a pointer button is released over the `target` entity.
pub type PointerUp = PointerEvent<Up>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
/// The inner [`PointerEvent`] type for [`PointerUp`].
pub struct Up {
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires when a pointer sends a pointer down event followed by a pointer up event, with the same
/// `target` entity for both events.
pub type PointerClick = PointerEvent<Click>;
/// The inner [`PointerEvent`] type for [`PointerClick`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Click {
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires while a pointer is moving over the `target` entity.
pub type PointerMove = PointerEvent<Move>;
/// The inner [`PointerEvent`] type for [`PointerMove`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Move;

/// Fires when the `target` entity receives a pointer down event followed by a pointer move event.
pub type PointerDragStart = PointerEvent<DragStart>;
/// The inner [`PointerEvent`] type for [`PointerDragStart`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragStart {
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires while the `target` entity is being dragged.
pub type PointerDrag = PointerEvent<Drag>;
/// The inner [`PointerEvent`] type for [`PointerDrag`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drag {
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
pub type PointerDragEnd = PointerEvent<DragEnd>;
/// The inner [`PointerEvent`] type for [`PointerDragEnd`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnd {
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires when a pointer dragging the `dragged` entity enters the `target` entity.
pub type PointerDragEnter = PointerEvent<DragEnter>;
/// The inner [`PointerEvent`] type for [`PointerDragEnter`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnter {
    /// The entity that was being dragged when the pointer entered the `target` entity.
    pub dragged: Entity,
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires while the `dragged` entity is being dragged over the `target` entity.
pub type PointerDragOver = PointerEvent<DragOver>;
/// The inner [`PointerEvent`] type for [`PointerDragOver`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragOver {
    /// The entity that was being dragged when the pointer was over the `target` entity.
    pub dragged: Entity,
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires when a pointer dragging the `dragged` entity leaves the `target` entity.
pub type PointerDragLeave = PointerEvent<DragLeave>;
/// The inner [`PointerEvent`] type for [`PointerDragLeave`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragLeave {
    /// The entity that was being dragged when the pointer left the `target` entity.
    pub dragged: Entity,
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Fires when a pointer drops the `dropped` entity onto the `target` entity.
pub type PointerDrop = PointerEvent<Drop>;
/// The inner [`PointerEvent`] type for [`PointerDrop`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drop {
    /// The entity that was dropped onto the `target` entity.
    pub dropped_entity: Entity,
    /// The pointer button associated with this drag event.
    pub button: PointerButton,
}

/// Generates pointer events from input data
pub fn pointer_events(
    // Input
    mut input_presses: EventReader<InputPress>,
    mut input_moves: EventReader<pointer::InputMove>,
    hover_map: Res<HoverMap>,
    previous_hover_map: Res<PreviousHoverMap>,
    // Output
    mut pointer_move: EventWriter<PointerMove>,
    mut pointer_over: EventWriter<PointerOver>,
    mut pointer_out: EventWriter<PointerOut>,
    mut pointer_up: EventWriter<PointerUp>,
    mut pointer_down: EventWriter<PointerDown>,
) {
    for move_event in input_moves.iter() {
        for hovered_entity in hover_map
            .get(&move_event.pointer_id())
            .iter()
            .flat_map(|h| h.iter())
        {
            pointer_move.send(PointerMove::new(
                &move_event.pointer_id(),
                hovered_entity,
                Move,
            ))
        }
    }

    for press_event in input_presses.iter() {
        // We use the previous hover map because we want to consider entities that just left the
        // entity. Without this, touch inputs would never send up events because they are lifted up
        // and leave the bounds of the entity at the same time.
        for hovered_entity in previous_hover_map
            .get(&press_event.pointer_id())
            .iter()
            .flat_map(|h| h.iter())
        {
            if let PressDirection::Up = press_event.direction() {
                pointer_up.send(PointerUp::new(
                    &press_event.pointer_id(),
                    hovered_entity,
                    Up {
                        button: press_event.button(),
                    },
                ))
            }
        }
        for hovered_entity in hover_map
            .get(&press_event.pointer_id())
            .iter()
            .flat_map(|h| h.iter())
        {
            if let PressDirection::Down = press_event.direction() {
                pointer_down.send(PointerDown::new(
                    &press_event.pointer_id(),
                    hovered_entity,
                    Down {
                        button: press_event.button(),
                    },
                ))
            }
        }
    }

    // If the entity is hovered...
    for (pointer_id, hovered_entity) in hover_map
        .iter()
        .flat_map(|(p, h)| h.iter().map(|h| (p.to_owned(), h)))
    {
        // ...but was not hovered last frame...
        if !previous_hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains(hovered_entity))
        {
            pointer_over.send(PointerOver::new(&pointer_id, hovered_entity, Over));
        }
    }

    // If the entity was hovered last frame...
    for (pointer_id, hovered_entity) in previous_hover_map
        .iter()
        .flat_map(|(p, h)| h.iter().map(|h| (p.to_owned(), h)))
    {
        // ...but is now not being hovered...
        if !hover_map
            .get(&pointer_id)
            .iter()
            .any(|e| e.contains(hovered_entity))
        {
            pointer_out.send(PointerOut::new(&pointer_id, hovered_entity, Out));
        }
    }
}

/// Uses pointer events to update [`PointerInteraction`] and [`Interaction`] components.
pub fn interactions_from_events(
    // Input
    mut pointer_over: EventReader<PointerOver>,
    mut pointer_out: EventReader<PointerOut>,
    mut pointer_up: EventReader<PointerUp>,
    mut pointer_down: EventReader<PointerDown>,
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

fn update_interactions<E: Clone + Send + Sync + Reflect>(
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
pub struct DragMap(pub HashMap<PointerId, Option<(Entity, PointerButton)>>);

/// Uses pointer events to determine when click and drag events occur.
pub fn send_click_and_drag_events(
    // Input
    mut pointer_down: EventReader<PointerDown>,
    mut pointer_up: EventReader<PointerUp>,
    mut pointer_move: EventReader<PointerMove>,
    mut input_move: EventReader<InputMove>,
    mut input_presses: EventReader<InputPress>,
    // Locals
    mut down_map: Local<HashMap<PointerId, Option<(Entity, PointerButton)>>>,
    // Output
    mut drag_map: ResMut<DragMap>,
    mut pointer_click: EventWriter<PointerClick>,
    mut pointer_drag_start: EventWriter<PointerDragStart>,
    mut pointer_drag_end: EventWriter<PointerDragEnd>,
    mut pointer_drag: EventWriter<PointerDrag>,
) {
    // Only triggers when over an entity
    for move_event in pointer_move.iter() {
        if let Some(Some((_, down_button))) = down_map.get(&move_event.pointer_id()) {
            let pointer_not_in_drag_map =
                matches!(drag_map.get(&move_event.pointer_id()), Some(None) | None);

            if pointer_not_in_drag_map {
                drag_map.insert(
                    move_event.pointer_id(),
                    Some((move_event.target(), *down_button)),
                );
                pointer_drag_start.send(PointerDragStart::new(
                    &move_event.pointer_id(),
                    &move_event.target(),
                    DragStart {
                        button: *down_button,
                    },
                ))
            }
        }
    }

    // Triggers during movement even if not over an entity
    for move_event in input_move.iter() {
        // if let Some(Some((down_entity, down_button))) = down_map.get(&move_event.pointer_id()) {
        if let Some(Some((drag_entity, drag_button))) = drag_map.get(&move_event.pointer_id()) {
            pointer_drag.send(PointerDrag::new(
                &move_event.pointer_id(),
                drag_entity,
                Drag {
                    button: *drag_button,
                },
            ))
        }
        // }
    }

    for up_event in pointer_up.iter() {
        if let Some(Some((down_entity, down_button))) = down_map.get(&up_event.pointer_id()) {
            if *down_entity == up_event.target() && up_event.event_data().button == *down_button {
                pointer_click.send(PointerClick::new(
                    &up_event.pointer_id(),
                    &up_event.target(),
                    Click {
                        button: *down_button,
                    },
                ));
            }
            if *down_button == up_event.event_data().button {
                down_map.insert(up_event.pointer_id(), None);
            }
        }

        if let Some(Some((drag_entity, drag_button))) = drag_map.get(&up_event.pointer_id()) {
            if *drag_button == up_event.event_data().button {
                pointer_drag_end.send(PointerDragEnd::new(
                    &up_event.pointer_id(),
                    drag_entity,
                    DragEnd {
                        button: *drag_button,
                    },
                ));
                drag_map.insert(up_event.pointer_id(), None);
            }
        }
    }

    for down_event in pointer_down.iter() {
        match down_map.get(&down_event.pointer_id()) {
            Some(None) | None => {
                down_map.insert(
                    down_event.pointer_id(),
                    Some((down_event.target(), down_event.event_data().button)),
                );
            }
            _ => (),
        }
    }

    for press_up in input_presses
        .iter()
        .filter(|e| e.direction() == pointer::PressDirection::Up)
    {
        if let Some(Some((drag_entity, drag_button))) =
            drag_map.get(&press_up.pointer_id()).copied()
        {
            if drag_button == press_up.button() {
                pointer_drag_end.send(PointerDragEnd::new(
                    &press_up.pointer_id(),
                    &drag_entity,
                    DragEnd {
                        button: drag_button,
                    },
                ));
                drag_map.insert(press_up.pointer_id(), None);
            }
        }
        if let Some(Some((_, down_button))) = down_map.get(&press_up.pointer_id()).copied() {
            if down_button == press_up.button() {
                down_map.insert(press_up.pointer_id(), None);
            }
        }
    }
}

/// Uses pointer events to determine when drag-over events occur
pub fn send_drag_over_events(
    // Input
    drag_map: Res<DragMap>,
    mut pointer_over: EventReader<PointerOver>,
    mut pointer_move: EventReader<PointerMove>,
    mut pointer_out: EventReader<PointerOut>,
    mut pointer_drag_end: EventReader<PointerDragEnd>,
    // Local
    mut drag_over_map: Local<HashMap<PointerId, HashSet<Entity>>>,
    // Output
    mut pointer_drag_enter: EventWriter<PointerDragEnter>,
    mut pointer_drag_over: EventWriter<PointerDragOver>,
    mut pointer_drag_leave: EventWriter<PointerDragLeave>,
    mut pointer_drop: EventWriter<PointerDrop>,
) {
    // Fire PointerDragEnter events.
    for over_event in pointer_over.iter() {
        if let Some(&Some((drag_entity, drag_button))) = drag_map.get(&over_event.pointer_id()) {
            if over_event.target() == drag_entity {
                // You can't drag an entity over itself
                continue;
            }
            let drag_entry = drag_over_map.entry(over_event.pointer_id()).or_default();
            drag_entry.insert(over_event.target());
            pointer_drag_enter.send(PointerDragEnter::new(
                &over_event.pointer_id(),
                &over_event.target(),
                DragEnter {
                    dragged: drag_entity,
                    button: drag_button,
                },
            ))
        }
    }
    // Fire PointerDragOver events.
    for move_event in pointer_move.iter() {
        if let Some(&Some((dragged, button))) = drag_map.get(&move_event.pointer_id()) {
            if move_event.target() == dragged {
                // You can't drag an entity over itself
                continue;
            }
            pointer_drag_over.send(PointerDragOver::new(
                &move_event.pointer_id(),
                &move_event.target(),
                DragOver { dragged, button },
            ))
        }
    }
    // Fire PointerDragLeave and PointerDrop events when the pointer stops dragging.
    for drag_end_event in pointer_drag_end.iter() {
        if let Some(drag_over_set) = drag_over_map.get_mut(&drag_end_event.pointer_id()) {
            for dragged_over in drag_over_set.drain() {
                pointer_drag_leave.send(PointerDragLeave::new(
                    &drag_end_event.pointer_id(),
                    &dragged_over,
                    DragLeave {
                        dragged: drag_end_event.target(),
                        button: drag_end_event.event_data().button,
                    },
                ));
                pointer_drop.send(PointerDrop::new(
                    &drag_end_event.pointer_id(),
                    &dragged_over,
                    Drop {
                        dropped_entity: drag_end_event.target(),
                        button: drag_end_event.event_data().button,
                    },
                ));
            }
        }
    }
    // Fire PointerDragLeave events when the pointer goes out of the target.
    for out_event in pointer_out.iter() {
        let out_pointer = &out_event.pointer_id();
        let out_target = &out_event.target();
        if let Some(dragged_over) = drag_over_map.get_mut(out_pointer) {
            if dragged_over.take(out_target).is_some() {
                if let Some(&Some((dragged, button))) = drag_map.get(out_pointer) {
                    pointer_drag_leave.send(PointerDragLeave::new(
                        out_pointer,
                        out_target,
                        DragLeave { dragged, button },
                    ))
                }
            }
        }
    }
}
