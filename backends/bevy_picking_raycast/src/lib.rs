//! A raycasting backend for `bevy_mod_picking` that uses `bevy_mod_raycast` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_mod_raycast::{
    system_param::{Raycast, RaycastVisibility},
    Ray3d,
};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_raycast`](crate) crate.
pub mod prelude {
    pub use crate::{RaycastBackend, RaycastPickCamera, RaycastPickTarget};
}

/// Adds the raycasting picking backend to your app.
#[derive(Clone)]
pub struct RaycastBackend;
impl Plugin for RaycastBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            (build_rays_from_pointers)
                .chain()
                .in_set(PickSet::PostInput),
        )
        .add_systems(
            PreUpdate,
            (
                bevy_mod_raycast::update_raycast::<RaycastPickingSet>,
                update_hits,
            )
                .chain()
                .in_set(PickSet::Backend),
        );
    }
}

/// This unit struct is used to tag the generic ray casting types
/// [`RaycastMesh`](bevy_mod_raycast::RaycastMesh) and
/// [`RaycastSource`](bevy_mod_raycast::RaycastSource).
#[derive(Reflect, Clone)]
pub struct RaycastPickingSet;

/// Marks an entity that should be pickable with [`bevy_mod_raycast`] ray casts.
pub type RaycastPickTarget = bevy_mod_raycast::RaycastMesh<RaycastPickingSet>;

/// Marks a camera that should be used for picking with [`bevy_mod_raycast`].
#[derive(Debug, Default, Clone, Component, Reflect)]
pub struct RaycastPickCamera {
    #[reflect(ignore)]
    /// Maps the pointers visible to this [`RaycastPickCamera`] to their corresponding ray. We need
    /// to create a map because many pointers may be visible to this camera.
    ray_map: HashMap<PointerId, Ray3d>,
}

impl RaycastPickCamera {
    /// Returns a map that defines the [`Ray3d`] associated with every [`PointerId`] that is on this
    /// [`RaycastPickCamera`]'s render target.
    pub fn ray_map(&self) -> &HashMap<PointerId, Ray3d> {
        &self.ray_map
    }
}

/// Builds rays and updates raycasting [`RaycastPickCamera`]s from [`PointerLocation`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(&PointerId, &PointerLocation)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut picking_cameras: Query<(&Camera, &GlobalTransform, &mut RaycastPickCamera)>,
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
            .filter(|(camera, _, _)| pointer_location.is_in_viewport(camera, &primary_window))
            .for_each(|(camera, transform, mut source)| {
                if let Some(ray) =
                    Ray3d::from_screenspace(pointer_location.position, camera, transform)
                {
                    source.ray_map.insert(*pointer_id, ray);
                }
            });
    }
}

/// Produces [`PointerHits`]s from [`RaycastSource`] intersections.
fn update_hits(
    pick_cameras: Query<(Entity, &Camera, &RaycastPickCamera)>,
    mut raycast: Raycast<RaycastPickingSet>,
    mut output_events: EventWriter<PointerHits>,
    pickables: Query<&Pickable>,
) {
    pick_cameras.iter().for_each(|(cam_entity, camera, map)| {
        for (&pointer, &ray) in map.ray_map().iter() {
            let settings = bevy_mod_raycast::system_param::RaycastSettings {
                visibility: RaycastVisibility::MustBeVisibleAndInView,
                filter: &|_| true, // Consider all entities in the raycasting set
                early_exit_test: &|entity_hit| {
                    pickables
                        .get(entity_hit)
                        .is_ok_and(|pickable| pickable.should_block_lower)
                },
            };
            let picks = raycast
                .cast_ray(ray, &settings)
                .iter()
                .map(|(entity, hit)| {
                    (
                        *entity,
                        HitData {
                            camera: cam_entity,
                            depth: hit.distance(),
                            position: Some(hit.position()),
                            normal: Some(hit.normal()),
                        },
                    )
                })
                .collect::<Vec<_>>();
            let order = camera.order as f32;
            if !picks.is_empty() {
                output_events.send(PointerHits {
                    pointer,
                    picks,
                    order,
                });
            }
        }
    });
}
