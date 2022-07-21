//! This module provides a simple interface for implementing a picking backend. A picking backend is
//! responsible for reading [`PointerPosition`](crate::input::PointerPosition) components, and
//! producing [`EntitiesUnderPointer`]s. The [`EntitiesUnderPointer`]s produced by a backend do
//! **not** need to be sorted or filtered, all that needs to be provided is an unordered list of
//! entities and their distance from the pointer into the screen (depth). Depth only needs to be
//! self-consistent with other [`EntitiesUnderPointer`]s in the same
//! [`PickLayer`](crate::focus::PickLayer).
//!
//! In plain English, a backend is provided the location of pointers, and is asked to provide a list
//! of all entities under those pointers.
//!
//! Because bevy_picking_core is very loosely coupled with its backends, you can mix and match as
//! many backends as you want. For example, You could use the `rapier` backend to raycast against
//! physics objects, a picking shader backend to pick non-physics meshes, and a custom backend for
//! your UI. The [`EntitiesUnderPointer`]s produced by these various backends will be combined,
//! sorted, and used as a homogeneous input for the picking systems.

use crate::PointerId;
use bevy::prelude::*;

/// An event produced by a picking backend, describing the entities under a pointer.
///
/// Some backends may only support providing the topmost entity; this is a valid limitation of some
/// backends. For example, a picking shader might only have data on the topmost rendered output from
/// its buffer.
#[derive(Debug, Clone)]
pub struct EntitiesUnderPointer {
    pub id: PointerId,
    pub over_list: Vec<PointerOverMetadata>,
}

/// Metadata for each entity this pointer is over.
#[derive(Debug, Clone)]
pub struct PointerOverMetadata {
    pub entity: Entity,
    pub depth: f32,
}
