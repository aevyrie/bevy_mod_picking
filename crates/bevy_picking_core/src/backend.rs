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
//! - Backends do not need to consider the [`Pickable`](crate::Pickable) component, though they may
//!   use it for optimization purposes. For example, a backend that traverses a spatial hierarchy
//!   may want to early exit if it intersects entity that blocks lower entities from being picked.

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
#[derive(Event, Debug, Clone)]
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
    ///
    /// ### Why is this an `f32`???
    ///
    /// Bevy UI is special in that it can share a camera with other things being rendered. in order
    /// to properly sort them, we need a way to make bevy_ui's order a tiny bit higher, like adding
    /// 0.5 to the order. We can't use integers, and we want users to be using camera.order by
    /// default, so this is the best solution at the moment.
    pub order: f32,
}

impl PointerHits {
    #[allow(missing_docs)]
    pub fn new(pointer: prelude::PointerId, picks: Vec<(Entity, HitData)>, order: f32) -> Self {
        Self {
            pointer,
            picks,
            order,
        }
    }
}

/// Holds data from a successful pointer hit test.
///
/// `depth` only needs to be self-consistent with other [`PointerHits`]s using the same
/// [`RenderTarget`](bevy::render::camera::RenderTarget).
#[derive(Clone, Debug, PartialEq, Reflect)]
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

impl HitData {
    #[allow(missing_docs)]
    pub fn new(camera: Entity, depth: f32, position: Option<Vec3>, normal: Option<Vec3>) -> Self {
        Self {
            camera,
            depth,
            position,
            normal,
        }
    }
}
