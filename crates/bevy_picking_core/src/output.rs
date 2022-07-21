//! Processes data from input and backends, producing interaction events.

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{input, PointerId};
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

pub type PointerEnter = PointerEvent<Enter>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Enter;

pub type PointerLeave = PointerEvent<Leave>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Leave;

pub type PointerDown = PointerEvent<Down>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Down;

pub type PointerUp = PointerEvent<Up>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Up;

pub type PointerClick = PointerEvent<Click>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Click;

pub type PointerMove = PointerEvent<Move>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Move;

pub type PointerCancel = PointerEvent<Cancel>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Cancel;

pub type PointerDragStart = PointerEvent<DragStart>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragStart;

pub type PointerDragEnd = PointerEvent<DragEnd>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct DragEnd;

pub type PointerDrag = PointerEvent<Drag>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Drag;

pub fn interactions_from_events(
    // Input
    mut pointer_over: EventReader<PointerEvent<Over>>,
    mut pointer_out: EventReader<PointerEvent<Out>>,
    mut pointer_up: EventReader<PointerEvent<Up>>,
    mut pointer_down: EventReader<PointerEvent<Down>>,
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

/// Sends click events when an entity receives a mouse down event followed by a mouse up event from
/// the same pointer and from within the same entity.
pub fn send_click_and_drag_events(
    // Input
    mut pointer_down: EventReader<PointerDown>,
    mut pointer_up: EventReader<PointerUp>,
    mut pointer_move: EventReader<PointerMove>,
    mut input_presses: EventReader<input::InputPress>,
    // Locals
    mut click_down: Local<HashMap<PointerId, Option<Entity>>>,
    mut drag_map: Local<HashMap<PointerId, Option<Entity>>>,
    // Output
    mut pointer_click: EventWriter<PointerClick>,
    mut pointer_drag_start: EventWriter<PointerDragStart>,
    mut pointer_drag_end: EventWriter<PointerDragEnd>,
    mut pointer_drag: EventWriter<PointerDrag>,
) {
    // The pointer moved and was already pressed
    for event in pointer_move.iter() {
        if let Some(Some(_)) = click_down.get(&event.id()) {
            if let Some(Some(drag_entity)) = drag_map.get(&event.id()) {
                pointer_drag.send(PointerDrag::new(&event.id(), drag_entity, Drag))
            } else {
                drag_map.insert(event.id(), Some(event.target()));
                pointer_drag_start.send(PointerDragStart::new(
                    &event.id(),
                    &event.target(),
                    DragStart,
                ))
            }
        }
    }

    for event in pointer_up.iter() {
        if let Some(Some(down_entity)) = click_down.get(&event.id()) {
            if *down_entity == event.target() {
                pointer_click.send(PointerClick::new(&event.id(), &event.target(), Click));
            }
            if let Some(Some(drag_entity)) = drag_map.get(&event.id()) {
                pointer_drag_end.send(PointerDragEnd::new(&event.id(), drag_entity, DragEnd));
            }
            drag_map.insert(event.id(), None);
        }
        click_down.insert(event.id(), None);
    }

    for event in pointer_down.iter() {
        click_down.insert(event.id(), Some(event.target()));
    }

    for press in input_presses.iter() {
        if press.press == input::PressStage::Up {
            click_down.insert(press.id, None);
        }
    }
}
