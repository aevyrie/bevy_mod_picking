//! This module provides a simple interface for implementing a picking backend. A picking backend is
//! responsible for reading [`PointerLocation`](crate::pointer::PointerLocation) components, and
//! producing [`EntitiesUnderPointer`] events. The [`EntitiesUnderPointer`] events produced by a
//! backend do **not** need to be sorted or filtered, all that is needed is an unordered list of
//! entities and their distance from the pointer into the screen (depth). Depth only needs to be
//! self-consistent with other [`EntitiesUnderPointer`]s in the same
//! [`RenderLayers`](bevy::render::view::RenderLayers).
//!
//! In plain English, a backend is provided the location of pointers, and is asked to provide a list
//! of entities under those pointers.
//!
//! Because bevy_picking_core is very loosely coupled with its backends, you can mix and match as
//! many backends as you want. For example, You could use the `rapier` backend to raycast against
//! physics objects, a picking shader backend to pick non-physics meshes, and a custom backend for
//! your UI. The [`EntitiesUnderPointer`]s produced by these various backends will be combined,
//! sorted, and used as a homogeneous input for the picking systems.

use bevy::prelude::*;

/// Common imports for implementing a picking backend.
pub mod prelude {
    pub use super::{EntitiesUnderPointer, EntityDepth};
    pub use crate::pointer::{PointerId, PointerLocation};
    pub use crate::PickStage;
}

/// An event produced by a picking backend, describing the entities under a pointer in an unordered
/// list.
///
/// Some backends may only support providing the topmost entity; this is a valid limitation of some
/// backends. For example, a picking shader might only have data on the topmost rendered output from
/// its buffer.
#[derive(Debug, Clone)]
pub struct EntitiesUnderPointer {
    /// ID of the pointer this event is for
    pub pointer: prelude::PointerId,
    /// An unordered collection of entities and their distance (depth) from the cursor.
    pub over_list: Vec<EntityDepth>,
}

/// An entity and its distance from the pointer (depth).
#[derive(Debug, Clone)]
pub struct EntityDepth {
    /// The entity under the pointer
    pub entity: Entity,
    /// The distance from the pointer to the entity into the screen, or depth.
    pub depth: f32,
}
