//! Processes data from input and backends, producing interaction events.

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{
    focus::HoverMap,
    pointer::{self, InputMove, InputPress, PointerId, PressStage},
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
pub trait ForwardedEvent: Event {
    /// Create a new event from [`EventData`].
    fn new<E: IsPointerEvent>(event_data: &mut PointerEventData<E>) -> Self;
}

/// An `EventListener` marks an entity, informing the [`event_bubbling`] system to run the
/// `on_event` function when an event of type `E` is being bubbled up the hierarchy and reaches this
/// entity.
#[derive(Component, Clone)]
pub struct EventListener<E: IsPointerEvent> {
    /// A function that is called when the event listener is triggered.
    on_event: fn(&mut Commands, &mut PointerEventData<E>),
}

impl<E: IsPointerEvent> EventListener<E> {
    /// Create an [`EventListener`] that will run the supplied `on_event` function with access to
    /// bevy [`Commands`].
    pub fn new_run_commands(on_event: fn(&mut Commands, &mut PointerEventData<E>)) -> Self {
        Self { on_event }
    }

    /// Create an [`EventListener`] that will send an event of type `F` when the listener is
    /// triggered, then continue to bubble the original event up this entity's hierarchy.
    pub fn new_forward_event<F: ForwardedEvent>() -> Self {
        Self {
            on_event: |commands: &mut Commands, event_data: &mut PointerEventData<E>| {
                let forwarded_event = F::new(event_data);
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
    pub fn new_forward_event_and_halt<F: ForwardedEvent>() -> Self {
        Self {
            on_event: |commands: &mut Commands, event_data: &mut PointerEventData<E>| {
                let forwarded_event = F::new(event_data);
                commands.add(|world: &mut World| {
                    let mut events = world.get_resource_or_insert_with(Events::<F>::default);
                    events.send(forwarded_event);
                });
                event_data.stop_bubbling();
            },
        }
    }
}

/// Extends the [`EntityCommands`] trait, allowing you to call these methods when spawning an
/// entity.
pub trait EventListenerCommands {
    /// Listens for events of type `E`. When found, an event of type `F` will be sent.
    ///
    /// # Usage
    ///
    /// This will send your custom `MyForwardedEvent`, when this entity receives a `PointerClick`. A
    /// helpful way to read this statement is "forward events of type `PointerClick` to events of
    /// type `MyForwardedEvent`"
    ///
    /// ```
    /// # struct MyForwardedEvent;
    /// # impl ForwardedEvent for MyForwardedEvent {
    /// #     fn new(_event_data: &mut EventData<impl IsPointerEvent>) -> Self {
    /// #         MyForwardedEvent
    /// #     }
    /// # }
    /// # fn(mut commands: Commands){
    /// commands
    ///     .spawn()
    ///     .forward_events::<PointerClick, MyForwardedEvent>();
    /// # }
    /// ```
    fn forward_events<E: IsPointerEvent, F: ForwardedEvent>(&mut self) -> &mut Self;
    /// Listens for events of type `E`. When found, an event of type `F` will be sent. Finally,
    /// bubbling will be halted. See [`event_bubbling`] for details on how bubbling works.
    ///
    /// Prefer using `forward_events` instead, unless you have a good reason to halt bubbling.
    fn forward_events_and_halt<E: IsPointerEvent, F: ForwardedEvent>(&mut self) -> &mut Self;
}

impl<'w, 's, 'a> EventListenerCommands for EntityCommands<'w, 's, 'a> {
    fn forward_events<E: IsPointerEvent, F: ForwardedEvent>(&mut self) -> &mut Self {
        self.commands().add(|world: &mut World| {
            world.init_resource::<Events<F>>();
        });
        self.insert(EventListener::<E>::new_forward_event::<F>());
        self
    }
    fn forward_events_and_halt<E: IsPointerEvent, F: ForwardedEvent>(&mut self) -> &mut Self {
        self.commands().add(|world: &mut World| {
            world.init_resource::<Events<F>>();
        });
        self.insert(EventListener::<E>::new_forward_event_and_halt::<F>());
        self
    }
}

/// Data from a pointer event, for use with [`EventListener`]s and event forwarding.
///
/// This is similar to the [`PointerEvent`] struct, except it also contains the event listener for
/// this event, as well as the ability to stop bubbling this event. When you forward an event, this
/// is the data that you can use to build your own custom, [`ForwardedEvent`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PointerEventData<E: IsPointerEvent> {
    /// The pointer involved in this event.
    id: PointerId,
    /// The entity that was listening for this event.
    listener: Entity,
    /// The entity that this event was originally triggered on.
    target: Entity,
    /// The inner event data, if any, for the specific event that was triggered.
    event: E::InnerEventType,
    /// Controls whether this event will continue to bubble up the entity hierarchy.
    bubble: Bubble,
}
impl<E: IsPointerEvent> PointerEventData<E> {
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

    /// When called, this will stop bubbling from continuing up the target entity's hierarchy. See
    /// [`event_bubbling`] for details.
    pub fn stop_bubbling(&mut self) {
        self.bubble = Bubble::Burst
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

/// This trait restricts the types of events that can be used as [`PointerEvent`]s.
pub trait IsPointerEvent: Send + Sync + 'static + Display + Clone {
    /// The inner event type of the [`PointerEvent`].
    type InnerEventType: Send + Sync + Clone;
}

/// Stores the common data needed for all `PointerEvent`s.
#[derive(Clone, Eq, PartialEq, Debug, Reflect)]
pub struct PointerEvent<E: Send + Sync + Clone + 'static + Reflect> {
    id: PointerId,
    target: Entity,
    event: E,
}
impl<E: Clone + Send + Sync + Reflect> IsPointerEvent for PointerEvent<E> {
    type InnerEventType = E;
}
impl<E: Clone + Send + Sync + 'static + Reflect> PointerEvent<E> {
    /// Create a new `PointerEvent`.
    pub fn new(id: &PointerId, target: &Entity, event: E) -> Self {
        Self {
            id: *id,
            target: *target,
            event,
        }
    }

    /// Get the [`PointerId`] of this event.
    pub fn id(&self) -> PointerId {
        self.id
    }

    /// Get the target entity of this event.
    pub fn target(&self) -> Entity {
        self.target
    }
}

//TODO: add a system that errors if a user adds the EventListener<PointerEnter/PointerLeave>
//components

/// Bubbles [`PointerEvent`]s of event type `E`.
///
/// Event bubbling makes it simple for specific entities to listen for specific events. When a
/// `PointerEvent` event is fired, `event_bubbling` will look for an `EventListener` on the event's
/// target entity, then walk up the hierarchy of the entity's ancestors, until a [`Bubble`]`::Pop`
/// is found or the root of the hierarchy is reached.
///
/// For every entity in the hierarchy, this system will look for an [`EventListener`]  matching the
/// event type `E`, and run the `on_event` function in the event listener.
///
/// Some `PointerEvent`s cannot be bubbled, and are instead sent to the entire hierarchy.
pub fn event_bubbling<E: Clone + Send + Sync + 'static + Reflect>(
    mut commands: Commands,
    mut events: EventReader<PointerEvent<E>>,
    listeners: Query<(Option<&EventListener<PointerEvent<E>>>, Option<&Parent>)>,
) {
    for event in events.iter() {
        let mut listener = event.target;
        while let Ok((event_listener, parent)) = listeners.get(listener) {
            match event_listener {
                Some(event_listener) => {
                    let mut event_data = PointerEventData {
                        id: event.id,
                        listener,
                        target: event.target,
                        event: event.event.clone(),
                        bubble: Bubble::default(),
                    };
                    (event_listener.on_event)(&mut commands, &mut event_data);
                    match event_data.bubble {
                        Bubble::Up => match parent {
                            Some(parent) => listener = **parent,
                            None => break, // Bubble reached the surface!
                        },
                        Bubble::Burst => break,
                    }
                }
                None => match parent {
                    Some(parent) => listener = **parent,
                    None => break, // Bubble reached the surface!
                },
            }
        }
    }
}
impl<E: Clone + Send + Sync + Reflect> std::fmt::Display for PointerEvent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Target: {:?}, Pointer: {:?}", self.target, self.id)
    }
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

/// TODO:
pub type PointerEnter = PointerEvent<Enter>;
/// The inner [`PointerEvent`] type for [`PointerEnter`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Enter;

/// TODO:
pub type PointerLeave = PointerEvent<Leave>;
/// The inner [`PointerEvent`] type for [`PointerLeave`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Leave;

/// Fires when a the pointer primary button is pressed over the `target` entity.
pub type PointerDown = PointerEvent<Down>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
/// The inner [`PointerEvent`] type for [`PointerDown`].
pub struct Down;

/// Fires when a the pointer primary button is released over the `target` entity.
pub type PointerUp = PointerEvent<Up>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
/// The inner [`PointerEvent`] type for [`PointerUp`].
pub struct Up;

/// Fires when a pointer sends a pointer down event followed by a pointer up event, with the same
/// `target` entity for both events.
pub type PointerClick = PointerEvent<Click>;
/// The inner [`PointerEvent`] type for [`PointerClick`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Click;

/// Fires while a pointer is moving over the `target` entity.
pub type PointerMove = PointerEvent<Move>;
/// The inner [`PointerEvent`] type for [`PointerMove`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Move;

///TODO:
pub type PointerCancel = PointerEvent<Cancel>;
/// The inner [`PointerEvent`] type for [`PointerCancel`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Cancel;

/// Fires when the `target` entity receives a pointer down event followed by a pointer move event.
pub type PointerDragStart = PointerEvent<DragStart>;
/// The inner [`PointerEvent`] type for [`PointerDragStart`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragStart;

/// Fires while the `target` entity is being dragged.
pub type PointerDrag = PointerEvent<Drag>;
/// The inner [`PointerEvent`] type for [`PointerDrag`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drag;

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
pub type PointerDragEnd = PointerEvent<DragEnd>;
/// The inner [`PointerEvent`] type for [`PointerDragEnd`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnd;

/// Fires when a pointer dragging some entity enters the `target` entity.
pub type PointerDragEnter = PointerEvent<DragEnter>;
/// The inner [`PointerEvent`] type for [`PointerDragEnter`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnter;

/// Fires while some entity is being dragged over the `target` entity.
pub type PointerDragOver = PointerEvent<DragOver>;
/// The inner [`PointerEvent`] type for [`PointerDragOver`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragOver;

/// Fires when a pointer dragging some entity leaves the `target` entity.
pub type PointerDragLeave = PointerEvent<DragLeave>;
/// The inner [`PointerEvent`] type for [`PointerDragLeave`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragLeave;

/// Fires when a pointer drops some entity onto the `target` entity.
pub type PointerDrop = PointerEvent<Drop>;
/// The inner [`PointerEvent`] type for [`PointerDrop`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drop;

/// Generates pointer events from input data
pub fn pointer_events(
    // Input
    pointers: Query<(&PointerId, &PointerInteraction)>, // <- what happened last frame
    mut input_presses: EventReader<pointer::InputPress>,
    mut pointer_move_in: EventReader<pointer::InputMove>,
    hover_map: Res<HoverMap>,
    // Output
    mut pointer_move: EventWriter<PointerMove>,
    mut pointer_over: EventWriter<PointerOver>,
    mut pointer_out: EventWriter<PointerOut>,
    mut pointer_up: EventWriter<PointerUp>,
    mut pointer_down: EventWriter<PointerDown>,
) {
    let input_presses: Vec<&InputPress> = input_presses.iter().collect();

    for event in pointer_move_in.iter() {
        for hover_entity in hover_map.get(&event.id()).iter().flat_map(|h| h.iter()) {
            pointer_move.send(PointerMove::new(&event.id(), hover_entity, Move))
        }
    }

    for (pointer_id, pointer_interaction) in pointers.iter() {
        let just_pressed = input_presses
            .iter()
            .filter_map(|click| (&click.id == pointer_id).then_some(click.press))
            .last();

        // If the entity is hovered...
        for hover_entity in hover_map.get(pointer_id).iter().flat_map(|h| h.iter()) {
            // ...but was not hovered last frame...
            if matches!(
                pointer_interaction.get(hover_entity),
                Some(Interaction::None) | None
            ) {
                pointer_over.send(PointerOver::new(pointer_id, hover_entity, Over));
            }

            match just_pressed {
                Some(PressStage::Down) => {
                    pointer_down.send(PointerDown::new(pointer_id, hover_entity, Down));
                }
                Some(PressStage::Up) => {
                    pointer_up.send(PointerUp::new(pointer_id, hover_entity, Up));
                }
                None => (),
            }
        }

        if let Some(hover_entities) = hover_map.get(pointer_id) {
            // If the entity was hovered last frame...
            for entity in pointer_interaction
                .iter()
                .filter_map(|(entity, interaction)| {
                    matches!(interaction, Interaction::Hovered | Interaction::Clicked)
                        .then_some(entity)
                })
            {
                // ...but is now not being hovered...
                if !hover_entities.contains(entity) {
                    if matches!(just_pressed, Some(PressStage::Up)) {
                        // ...the pointer is considered just up on this entity even though it was
                        // not hovering the entity this frame
                        pointer_up.send(PointerUp::new(pointer_id, entity, Up));
                    }
                    pointer_out.send(PointerOut::new(pointer_id, entity, Out));
                }
            }
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
    mut entities: Query<&mut Interaction>,
) {
    for event in pointer_over.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut entities);
    }
    for event in pointer_out.iter() {
        update_interactions(event, Interaction::None, &mut pointers, &mut entities);
    }
    for event in pointer_down.iter() {
        update_interactions(event, Interaction::Clicked, &mut pointers, &mut entities);
    }
    for event in pointer_up.iter() {
        update_interactions(event, Interaction::Hovered, &mut pointers, &mut entities);
    }
}

fn update_interactions<E: Clone + Send + Sync + Reflect>(
    event: &PointerEvent<E>,
    new_interaction: Interaction,
    pointer_interactions: &mut Query<(&PointerId, &mut PointerInteraction)>,
    entity_interactions: &mut Query<&mut Interaction>,
) {
    pointer_interactions
        .iter_mut()
        .find_map(|(id, interaction)| (*id == event.id).then_some(interaction))
        .and_then(|mut interaction_map| interaction_map.insert(event.target, new_interaction));
    entity_interactions.for_each_mut(|mut interaction| *interaction = new_interaction);
}

/// Maps pointers to the entities they are dragging.
#[derive(Debug, Deref, DerefMut, Default)]
pub struct DragMap(pub HashMap<PointerId, Option<Entity>>);

/// Uses pointer events to determine when click and drag events occur.
pub fn send_click_and_drag_events(
    // Input
    mut pointer_down: EventReader<PointerDown>,
    mut pointer_up: EventReader<PointerUp>,
    mut pointer_move: EventReader<PointerMove>,
    mut input_move: EventReader<InputMove>,
    mut input_presses: EventReader<pointer::InputPress>,
    // Locals
    mut down_map: Local<HashMap<PointerId, Option<Entity>>>,
    // Output
    mut drag_map: ResMut<DragMap>,
    mut pointer_click: EventWriter<PointerClick>,
    mut pointer_drag_start: EventWriter<PointerDragStart>,
    mut pointer_drag_end: EventWriter<PointerDragEnd>,
    mut pointer_drag: EventWriter<PointerDrag>,
) {
    // Only triggers when over an entity
    for move_event in pointer_move.iter() {
        if let Some(Some(_)) = down_map.get(&move_event.id()) {
            if matches!(drag_map.get(&move_event.id()), Some(None) | None) {
                drag_map.insert(move_event.id(), Some(move_event.target()));
                pointer_drag_start.send(PointerDragStart::new(
                    &move_event.id(),
                    &move_event.target(),
                    DragStart,
                ))
            }
        }
    }

    // Triggers during movement even if not over an entity
    for move_event in input_move.iter() {
        if let Some(Some(_)) = down_map.get(&move_event.id()) {
            if let Some(Some(drag_entity)) = drag_map.get(&move_event.id()) {
                pointer_drag.send(PointerDrag::new(&move_event.id(), drag_entity, Drag))
            }
        }
    }

    for event in pointer_up.iter() {
        if let Some(Some(down_entity)) = down_map.get(&event.id()) {
            if *down_entity == event.target() {
                pointer_click.send(PointerClick::new(&event.id(), &event.target(), Click));
            }
            if let Some(Some(drag_entity)) = drag_map.get(&event.id()) {
                pointer_drag_end.send(PointerDragEnd::new(&event.id(), drag_entity, DragEnd));
            }
            drag_map.insert(event.id(), None);
        }
        down_map.insert(event.id(), None);
    }

    for event in pointer_down.iter() {
        down_map.insert(event.id(), Some(event.target()));
    }

    for press in input_presses.iter() {
        if press.press == pointer::PressStage::Up {
            if let Some(Some(drag_entity)) = drag_map.insert(press.id, None) {
                pointer_drag_end.send(PointerDragEnd::new(&press.id, &drag_entity, DragEnd));
            }
            down_map.insert(press.id, None);
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
    mut drag_over_map: Local<HashMap<PointerId, Option<Entity>>>,
    // Output
    mut pointer_drag_enter: EventWriter<PointerDragEnter>,
    mut pointer_drag_over: EventWriter<PointerDragOver>,
    mut pointer_drag_leave: EventWriter<PointerDragLeave>,
    mut pointer_drop: EventWriter<PointerDrop>,
) {
    // Fire PointerDragEnter events.
    for over_event in pointer_over.iter() {
        if let Some(Some(dragged)) = drag_map.get(&over_event.id()) {
            if &over_event.target() != dragged {
                drag_over_map.insert(over_event.id(), Some(over_event.target()));
                pointer_drag_enter.send(PointerDragEnter::new(
                    &over_event.id(),
                    &over_event.target(),
                    DragEnter,
                ))
            }
        }
    }
    // Fire PointerDragOver events.
    for move_event in pointer_move.iter() {
        if let Some(Some(dragged)) = drag_map.get(&move_event.id()) {
            if &move_event.target() != dragged {
                pointer_drag_over.send(PointerDragOver::new(
                    &move_event.id(),
                    &move_event.target(),
                    DragOver,
                ))
            }
        }
    }
    // Fire PointerDragLeave events when the pointer goes out of the target.
    for out_event in pointer_out.iter() {
        if let Some(dragged_over) = drag_over_map.get_mut(&out_event.id()) {
            if Some(out_event.target()) == *dragged_over {
                *dragged_over = None;
                pointer_drag_leave.send(PointerDragLeave::new(
                    &out_event.id(),
                    &out_event.target(),
                    DragLeave,
                ))
            }
        }
    }
    // Fire PointerDragLeave and PointerDrop events when the pointer stops dragging.
    for drag_end_event in pointer_drag_end.iter() {
        if let Some(maybe_dragged_over) = drag_over_map.get_mut(&drag_end_event.id()) {
            if let Some(dragged_over) = *maybe_dragged_over {
                *maybe_dragged_over = None;
                pointer_drag_leave.send(PointerDragLeave::new(
                    &drag_end_event.id(),
                    &dragged_over,
                    DragLeave,
                ));
                pointer_drop.send(PointerDrop::new(&drag_end_event.id(), &dragged_over, Drop));
            }
        }
    }
}
