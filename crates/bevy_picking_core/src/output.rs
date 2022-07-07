//! Processes data from input and backend, then outputs interaction states.

use crate::PointerId;
use bevy::{prelude::*, utils::HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct PointerInteractionEvent {
    pub pick_entity: Entity,
    pub id: PointerId,
    pub event: Just,
}
impl std::fmt::Display for PointerInteractionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            pick_entity,
            id,
            event,
        } = self;
        write!(
            f,
            "Event::Interaction::{event:?}::{id:?} {pick_entity:.15?}"
        )
    }
}
impl PointerInteractionEvent {
    pub fn new(pick_entity: Entity, pointer: PointerId, event: Just) -> Self {
        Self {
            pick_entity,
            id: pointer,
            event,
        }
    }
}

/// An event that wraps selection and hover events
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub enum Just {
    /// Like `mouseover`    
    Entered,
    /// Like `mouseout`
    Exited,
    /// Like `mousedown`
    Down,
    /// Like `mouseup`
    Up,
    /// Like `click`
    Clicked,
    /// Like `mousemove`
    Moved,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct PickInteraction {
    pub(crate) map: HashMap<PointerId, Interaction>,
}
impl PickInteraction {
    pub fn is_hovered_by(&self, pointer: &PointerId) -> bool {
        self.map
            .get(pointer)
            .filter(|i| matches!(i, Interaction::Hovered) || matches!(i, Interaction::Clicked))
            .is_some()
    }

    pub fn is_clicked_by(&self, pointer: &PointerId) -> bool {
        self.map
            .get(pointer)
            .filter(|i| matches!(i, Interaction::Clicked))
            .is_some()
    }

    pub fn is_hovered_any(&self) -> bool {
        self.map
            .values()
            .any(|i| matches!(&i, Interaction::Hovered) || matches!(&i, Interaction::Clicked))
    }

    pub fn is_clicked_any(&self) -> bool {
        self.map
            .values()
            .any(|i| matches!(&i, Interaction::Clicked))
    }
}
