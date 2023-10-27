//! A raycasting backend for `bevy_mod_picking` that uses `rapier` for raycasting.
//!
//! # Usage
//!
//! This backend requires you mark any cameras that will be used for raycasting with
//! [`RapierPickCamera`]. If a pointer passes through this camera's render target, it will
//! automatically shoot rays into the rapier scene and will be able to pick things.
//!
//! To ignore an entity, you can add [`Pickable::IGNORE`] to it, and it will be ignored during
//! raycasting.
//!
//! ## Limitations
//!
//! Because raycasting is expensive, only the closest intersection will be reported. This means that
//! unlike some UI, you cannot hover multiple rapier objects with a single pointer by configuring
//! the [`Pickable`] component to not block lower elements but still emit events. As mentioned
//! above, all that is supported is completely ignoring an entity with [`Pickable::IGNORE`].
//!
//! This is probably not a meaningful limitation, as the feature is usually only used in UI where
//! you might want a pointer to be able to pick multiple elements that are on top of each other. If
//! are trying to build a UI out of rapier entities, beware, I suppose.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::Ray;
use bevy_reflect::prelude::*;
use bevy_render::prelude::*;
use bevy_transform::prelude::*;
use bevy_utils::HashMap;
use bevy_window::PrimaryWindow;

use bevy_picking_core::backend::prelude::*;
use bevy_rapier3d::prelude::*;

/// Commonly used imports.
pub mod prelude {
    pub use crate::{RapierBackend, RapierPickCamera};
}

/// Adds the `rapier` raycasting picking backend to your app.
#[derive(Clone)]
pub struct RapierBackend;
impl Plugin for RapierBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (build_rays_from_pointers, update_hits)
                .chain()
                .in_set(PickSet::Backend),
        );
    }
}

/// Marks a camera that should be used for rapier raycast picking.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct RapierPickCamera {
    #[reflect(ignore)]
    /// Maps the pointers visible to this [`RapierPickCamera`] to their corresponding ray. We need
    /// to create a map because many pointers may be visible to this camera.
    ray_map: HashMap<PointerId, Ray>,
}

impl RapierPickCamera {
    /// Returns a map that defines the [`Ray`] associated with every [`PointerId`] that is on this
    /// [`RapierPickCamera`]'s render target.
    pub fn ray_map(&self) -> &HashMap<PointerId, Ray> {
        &self.ray_map
    }
}

/// Updates all picking [`Ray`]s with [`PointerLocation`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(&PointerId, &PointerLocation)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut picking_cameras: Query<(&Camera, &GlobalTransform, &mut RapierPickCamera)>,
) {
    picking_cameras.iter_mut().for_each(|(_, _, mut pick_cam)| {
        pick_cam.ray_map.clear();
    });
    for (pointer_id, pointer_location) in &pointers {
        let pointer_location = match pointer_location.location() {
            Some(l) => l,
            None => continue,
        };
        picking_cameras
            .iter_mut()
            .filter(|(camera, _, _)| {
                camera.is_active && pointer_location.is_in_viewport(camera, &primary_window)
            })
            .for_each(|(camera, transform, mut source)| {
                let mut viewport_pos = pointer_location.position;
                if let Some(viewport) = &camera.viewport {
                    viewport_pos -= viewport.physical_position.as_vec2();
                }
                if let Some(ray) = camera.viewport_to_world(transform, viewport_pos) {
                    source.ray_map.insert(*pointer_id, ray);
                }
            });
    }
}

/// Produces [`PointerHits`]s from [`RapierPickRay`] intersections.
fn update_hits(
    rapier_context: Option<Res<RapierContext>>,
    targets: Query<&Pickable>,
    mut sources: Query<(Entity, &Camera, &mut RapierPickCamera)>,
    mut output: EventWriter<PointerHits>,
) {
    let rapier_context = match rapier_context {
        Some(c) => c,
        None => return,
    };

    sources
        .iter_mut()
        .flat_map(|(entity, camera, source)| {
            source
                .ray_map
                .iter()
                .map(|(pointer, ray)| (entity, camera.order, *pointer, *ray))
                .collect::<Vec<_>>()
        })
        .filter_map(|(cam_entity, cam_order, pointer, ray)| {
            rapier_context
                .cast_ray_and_get_normal(
                    ray.origin,
                    ray.direction,
                    f32::MAX,
                    true,
                    QueryFilter::new().predicate(&|entity| {
                        targets
                            .get(entity)
                            .map(|pickable| *pickable != Pickable::IGNORE)
                            .unwrap_or(true)
                    }),
                )
                .map(|(target, intersection)| {
                    (cam_entity, cam_order, pointer, target, intersection)
                })
        })
        .for_each(|(cam_entity, cam_order, pointer, target, hit)| {
            let hit = HitData::new(cam_entity, hit.toi, Some(hit.point), Some(hit.normal));
            let order = cam_order as f32;
            output.send(PointerHits::new(pointer, vec![(target, hit)], order));
        });
}
