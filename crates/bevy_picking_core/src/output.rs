//! Processes data from input and backends, producing interaction events.

use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::PointerId;
use bevy::{prelude::*, utils::HashMap};

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

pub trait IsPointerEventInner: Send + Sync + 'static {}

#[derive(Component, Clone)]
pub struct EventListener<E: IsPointerEvent> {
    /// Called when the event listener is triggered.
    on_event: fn(&mut Commands, PointerEventData) -> Bubble,
    spooky: PhantomData<E>,
}

impl<E: IsPointerEvent> EventListener<E> {
    pub fn new(on_event: fn(&mut Commands, PointerEventData) -> Bubble) -> Self {
        Self {
            on_event,
            spooky: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PointerEventData {
    /// The entity that was listening for this event.
    pub listener: Entity,
    /// The entity that this event was initially triggered on.
    pub target: Entity,
}

/// Should the event bubble up to the entity's parent, or halt?
#[derive(Default)]
pub enum Bubble {
    /// This event will bubble up to its parent.
    #[default]
    Up,
    /// Stops this event from bubbling to the next parent.
    Burst,
}

pub trait IsPointerEvent: Send + Sync + 'static + Display {}

#[derive(Debug, Clone)]
pub struct PointerEvent<E: IsPointerEventInner> {
    id: PointerId,
    target: Entity,
    spooky: PhantomData<E>,
}
impl<E: IsPointerEventInner> IsPointerEvent for PointerEvent<E> {}
impl<E: IsPointerEventInner> PointerEvent<E> {
    pub fn new(id: &PointerId, target: &Entity) -> Self {
        Self {
            id: *id,
            target: *target,
            spooky: PhantomData,
        }
    }

    pub fn id(&self) -> PointerId {
        self.id
    }

    pub fn target(&self) -> Entity {
        self.target
    }

    pub fn event_bubbling(
        mut commands: Commands,
        mut events: EventReader<PointerEvent<E>>,
        listeners: Query<(Option<&EventListener<PointerEvent<E>>>, Option<&Parent>)>,
    ) {
        for event in events.iter() {
            let target = event.target;
            let mut listener = event.target;
            while let Ok((event_listener, parent)) = listeners.get(listener) {
                match event_listener {
                    Some(event_listener) => {
                        match (event_listener.on_event)(
                            &mut commands,
                            PointerEventData { listener, target },
                        ) {
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
}
impl<E: IsPointerEventInner> std::fmt::Display for PointerEvent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Target: {:?}, Pointer: {:?}", self.target, self.id)
    }
}

pub type PointerOver = PointerEvent<Over>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Over {}
impl IsPointerEventInner for Over {}

pub type PointerOut = PointerEvent<Out>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Out {}
impl IsPointerEventInner for Out {}

pub type PointerEnter = PointerEvent<Enter>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Enter {}
impl IsPointerEventInner for Enter {}

pub type PointerLeave = PointerEvent<Leave>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Leave {}
impl IsPointerEventInner for Leave {}

pub type PointerDown = PointerEvent<Down>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Down {}
impl IsPointerEventInner for Down {}

pub type PointerUp = PointerEvent<Up>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Up {}
impl IsPointerEventInner for Up {}

pub type PointerClick = PointerEvent<Click>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Click {}
impl IsPointerEventInner for Click {}

pub type PointerMove = PointerEvent<Move>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Move {}
impl IsPointerEventInner for Move {}

pub type PointerCancel = PointerEvent<Cancel>;
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Cancel {}
impl IsPointerEventInner for Cancel {}

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct PickInteraction {
    pub(crate) inner: Interaction,
}
impl PickInteraction {
    pub fn is_hovered(&self) -> bool {
        matches!(self.inner, Interaction::Hovered | Interaction::Clicked)
    }

    pub fn is_pressed(&self) -> bool {
        matches!(self.inner, Interaction::Clicked)
    }

    pub fn update(
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
            Self::update_interactions(event, Interaction::Hovered, &mut pointers, &mut entities);
        }
        for event in pointer_out.iter() {
            Self::update_interactions(event, Interaction::None, &mut pointers, &mut entities);
        }
        for event in pointer_down.iter() {
            Self::update_interactions(event, Interaction::Clicked, &mut pointers, &mut entities);
        }
        for event in pointer_up.iter() {
            Self::update_interactions(event, Interaction::Hovered, &mut pointers, &mut entities);
        }
    }

    fn update_interactions<E: IsPointerEventInner>(
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
}
