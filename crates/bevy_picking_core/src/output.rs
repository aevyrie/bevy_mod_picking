//! Processes data from input and backends, producing interaction events.

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{
    input::{self, InputMove},
    PointerId,
};
use bevy::{
    ecs::{event::Event, system::EntityCommands},
    prelude::*,
    utils::HashMap,
};

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

pub trait EventFrom: Event {
    fn new(event_data: &mut EventData<impl IsPointerEvent>) -> Self;
}

#[derive(Component, Clone)]
pub struct EventListener<E: IsPointerEvent> {
    /// Called when the event listener is triggered.
    on_event: fn(&mut Commands, &mut EventData<E>),
}

impl<E: IsPointerEvent> EventListener<E> {
    pub fn run_command(on_event: fn(&mut Commands, &mut EventData<E>)) -> Self {
        Self { on_event }
    }
    pub fn forward_event<F: EventFrom>() -> Self {
        Self {
            on_event: |commands: &mut Commands, event_data: &mut EventData<E>| {
                let forwarded_event = F::new(event_data);
                commands.add(|world: &mut World| {
                    let mut events = world.get_resource_or_insert_with(Events::<F>::default);
                    events.send(forwarded_event);
                });
            },
        }
    }
    pub fn forward_event_and_break<F: EventFrom>() -> Self {
        Self {
            on_event: |commands: &mut Commands, event_data: &mut EventData<E>| {
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

pub trait EventListenerCommands {
    fn forward_events<E: IsPointerEvent, F: EventFrom>(&mut self) -> &mut Self;
    fn forward_events_and_break<E: IsPointerEvent, F: EventFrom>(&mut self) -> &mut Self;
}

impl<'w, 's, 'a> EventListenerCommands for EntityCommands<'w, 's, 'a> {
    fn forward_events<E: IsPointerEvent, F: EventFrom>(&mut self) -> &mut Self {
        self.commands().add(|world: &mut World| {
            world.init_resource::<Events<F>>();
        });
        self.insert(EventListener::<E>::forward_event::<F>());
        self
    }
    fn forward_events_and_break<E: IsPointerEvent, F: EventFrom>(&mut self) -> &mut Self {
        self.commands().add(|world: &mut World| {
            world.init_resource::<Events<F>>();
        });
        self.insert(EventListener::<E>::forward_event_and_break::<F>());
        self
    }
}

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
    /// Controls whether this event will continue to bubble up the entity hierarchy.
    bubble: Bubble,
}
impl<E: IsPointerEvent> EventData<E> {
    pub fn id(&self) -> PointerId {
        self.id
    }

    pub fn listener(&self) -> Entity {
        self.listener
    }

    pub fn target(&self) -> Entity {
        self.target
    }

    pub fn event(&self) -> &E::InnerEventType {
        &self.event
    }

    pub fn stop_bubbling(&mut self) {
        self.bubble = Bubble::Burst
    }
}

/// Should the event bubble up to the entity's parent, or halt?
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum Bubble {
    /// This event will bubble up to its parent.
    #[default]
    Up,
    /// Stops this event from bubbling to the next parent.
    Burst,
}

pub trait IsPointerEvent: Send + Sync + 'static + Display + Clone {
    type InnerEventType;
}

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
    pub fn new(id: &PointerId, target: &Entity, event: E) -> Self {
        Self {
            id: *id,
            target: *target,
            event,
        }
    }

    pub fn id(&self) -> PointerId {
        self.id
    }

    pub fn target(&self) -> Entity {
        self.target
    }
}

//TODO: add a system that errors if a user adds the EventListener<PointerEnter/PointerLeave>
//components

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
                    let mut event_data = EventData {
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

pub type PointerOver = PointerEvent<Over>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Over;

pub type PointerOut = PointerEvent<Out>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Out;

// TODO:
pub type PointerEnter = PointerEvent<Enter>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Enter;

//TODO:
pub type PointerLeave = PointerEvent<Leave>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Leave;

pub type PointerDown = PointerEvent<Down>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Down;

pub type PointerUp = PointerEvent<Up>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Up;

/// Fires when a pointer sends a mouse down event followed by a mouse up event, with the same
/// `target` entity for both events.
pub type PointerClick = PointerEvent<Click>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Click;

/// Fires while a pointer is moving over the `target` entity.
pub type PointerMove = PointerEvent<Move>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Move;

//TODO:
pub type PointerCancel = PointerEvent<Cancel>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Cancel;

/// Fires when the `target` entity receives a pointer down event followed by a pointer move event.
pub type PointerDragStart = PointerEvent<DragStart>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragStart;

/// Fires while the `target` entity is being dragged.
pub type PointerDrag = PointerEvent<Drag>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drag;

/// Fires when a pointer is dragging the `target` entity and a pointer up event is received.
pub type PointerDragEnd = PointerEvent<DragEnd>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnd;

/// Fires when a pointer dragging some entity enters the `target` entity.
pub type PointerDragEnter = PointerEvent<DragEnter>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnter;

/// Fires while some entity is being dragged over the `target` entity.
pub type PointerDragOver = PointerEvent<DragOver>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragOver;

/// Fires when a pointer dragging some entity leaves the `target` entity.
pub type PointerDragLeave = PointerEvent<DragLeave>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragLeave;

/// Fires when a pointer drops some entity onto the `target` entity.
pub type PointerDrop = PointerEvent<Drop>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drop;

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
    mut input_presses: EventReader<input::InputPress>,
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
        if press.press == input::PressStage::Up {
            down_map.insert(press.id, None);
        }
    }
}

// Uses pointer events to determine when drag-over events occur
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
