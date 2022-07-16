use crate::{
    backend::CursorOver,
    input::{CursorClick, CursorId, CursorLocation},
};
use bevy::prelude::*;

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
    Click,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct CursorEvent {
    pub entity: Entity,
    pub cursor: CursorId,
    pub event: Just,
}

impl CursorEvent {
    pub fn new(entity: Entity, cursor: CursorId, event: Just) -> Self {
        Self {
            entity,
            cursor,
            event,
        }
    }
}