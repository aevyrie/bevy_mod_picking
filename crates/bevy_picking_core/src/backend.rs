//! This module provides a simple interface for implementing a picking backend.
//!
//! Because bevy_picking_core is very loosely coupled with its backends, you can mix and match as
//! many backends as you want. For example, You could use the `rapier` backend to raycast against
//! physics objects, a picking shader backend to pick non-physics meshes, and a custom backend for
//! your UI. The [`PointerHits`]s produced by these various backends will be combined, sorted, and
//! used as a homogeneous input for the picking systems that consume these events.
//!
//! ## Implementation
//!
//! - A picking backend only has one job: reading
//! [`PointerLocation`](crate::pointer::PointerLocation) components, checking
//! [`Pickable`](crate::Pickable) entities for hits, and producing [`PointerHits`] events. In plain
//! English, a backend is provided the location of pointers, and is asked to provide a list of
//! entities under those pointers.
//!
//! - The [`PointerHits`] events produced by a backend do **not** need to be sorted or filtered, all
//! that is needed is an unordered list of entities and their [`HitData`].
//!
//! - **Backends should only pick entities with the [`Pickable`](crate::Pickable) component.**

use bevy::prelude::*;

/// Common imports for implementing a picking backend.
pub mod prelude {
    pub use super::{HitData, PointerHits};
    pub use crate::{
        pointer::{PointerId, PointerLocation},
        PickSet, Pickable,
    };
}

/// An event produced by a picking backend after it has run its hit tests, describing the entities
/// under a pointer.
///
/// Some backends may only support providing the topmost entity; this is a valid limitation of some
/// backends. For example, a picking shader might only have data on the topmost rendered output from
/// its buffer.
#[derive(Debug, Clone, Event)]
pub struct PointerHits {
    /// The pointer associated with this hit test.
    pub pointer: prelude::PointerId,
    /// An unordered collection of entities and their distance (depth) from the cursor.
    pub picks: Vec<(Entity, HitData)>,
    /// Set the order of this group of picks. Normally, this is the [`Camera::order`].
    ///
    /// Used to allow multiple `PointerHits` submitted for the same pointer to be ordered.
    /// `PointerHits` with a higher `order` will be checked before those with a lower `order`,
    /// regardless of the depth of each entity pick.
    ///
    /// In other words, when pick data is coalesced across all backends, the data is grouped by
    /// pointer, then sorted by order, and checked sequentially, sorting each `PointerHits` by
    /// entity depth. Events with a higher `order` are effectively on top of events with a lower
    /// order.
    pub order: isize,
}

/// Holds data from a successful pointer hit test.
///
/// `depth` only needs to be self-consistent with other [`PointerHits`]s using the same
/// [`RenderTarget`](bevy::render::camera::RenderTarget).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct HitData {
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
