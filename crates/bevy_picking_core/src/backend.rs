//! This module provides a simple interface for implementing a picking backend.
//!
//! A picking backend is responsible for reading
//! [`PointerLocation`](crate::pointer::PointerLocation) components, and producing
//! [`EntitiesUnderPointer`] events.
//!
//! The [`EntitiesUnderPointer`] events produced by a backend do **not** need to be sorted or
//! filtered, all that is needed is an unordered list of entities and their distance from the
//! pointer into the screen (depth). Depth only needs to be self-consistent with other
//! [`EntitiesUnderPointer`]s using the same [`RenderTarget`](bevy::render::camera::RenderTarget).
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
    pub use super::{EntitiesUnderPointer, PickData, PickingBackend};
    pub use crate::{
        pointer::{PointerId, PointerLocation},
        PickSet,
    };
}

/// Implement this trait for a group of plugins to make them useable as a picking backend.
pub trait PickingBackend: bevy::app::Plugin {}

impl Plugin for Box<dyn PickingBackend> {
    fn build(&self, app: &mut App) {
        (**self).build(app);
    }
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
    pub picks: Vec<(Entity, PickData)>,
    /// Set the order of this group of picks. Normally, this is the [`Camera::order`].
    ///
    /// Used to allow multiple `EntitiesUnderPointer` submitted for the same pointer to be ordered.
    /// `EntitiesUnderPointer` with a higher `order` will be checked before those with a lower
    /// `order`, regardless of the depth of each entity pick.
    ///
    /// In other words, when pick data is coalesced across all backends, the data is grouped by
    /// pointer, then sorted by order, and checked sequentially, sorting each `EntitiesUnderPointer`
    /// by entity depth. Events with a higher `order` are effectively on top of events with a lower
    /// order.
    pub order: isize,
}

/// Holds data about a pick intersection.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct PickData {
    /// The camera entity used to detect this hit. Useful when you need to find the ray that was
    /// casted for this hit when using a raycasting backend.
    pub camera: Entity,
    /// The distance from the pointer to the entity into the screen, or depth.
    pub depth: f32,
    /// The position of the intersection in the world, if the data is available from the backend.
    pub position: Option<Vec3>,
    /// The normal vector of the hit test, if the data is available from the backend.
    pub normal: Option<Vec3>,
}
