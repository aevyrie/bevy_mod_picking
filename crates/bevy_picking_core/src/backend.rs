//! A picking backend is responsible for reading [`crate::input::PointerPosition`]s, and producing
//! [`PointerOverEvent`]s. The [`PointerOverEvent`]s produced by a backend do not need to be sorted or
//! filtered, all that needs to be provided is an unordered list of entities and their distance from
//! the pointer into the screen (depth).
//!
//! Depth only needs to be self-consistent with other [`PointerOverEvent`]s in the same
//! [`crate::focus::PickLayer`].
//!
//! Because bevy_picking_core is very loosely coupled with its backends, you can mix and match as
//! many backends as you want. For example, You could use the `rapier` backend to raycast against
//! physics objects, a picking shader backend to pick non-physics meshes, and a custom backend for
//! your UI.

use crate::PointerId;
use bevy::prelude::*;

/// An event containing a point and the entities the pointer is over.
#[derive(Debug, Clone)]
pub struct PointerOverEvent {
    pub id: PointerId,
    pub over_list: Vec<PointerOverMetadata>,
}
impl std::fmt::Display for PointerOverEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event::Over::{:?} {:?}", self.id, self.over_list)
    }
}

#[derive(Debug, Clone)]
pub struct PointerOverMetadata {
    pub entity: Entity,
    pub depth: f32,
}
