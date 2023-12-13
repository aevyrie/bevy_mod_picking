//! Types and systems for constructing rays from cameras and pointers.

use crate::backend::prelude::{PointerId, PointerLocation};
use bevy_ecs::prelude::*;
use bevy_math::Ray;
use bevy_reflect::Reflect;
use bevy_render::camera::Camera;
use bevy_transform::prelude::GlobalTransform;
use bevy_utils::HashMap;
use bevy_window::PrimaryWindow;

/// Identifies a ray constructed from some (pointer, camera) combination.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct RayId {
    /// The camera whose projection was used to calculate the ray.
    pub camera: Entity,
    /// The pointer whose pixel coordinates were used to calculate the ray.
    pub pointer: PointerId,
}

impl RayId {
    /// Construct a [`RayId`].
    pub fn new(camera: Entity, pointer: PointerId) -> Self {
        Self { camera, pointer }
    }
}

/// A map from [`RayId`] to [`Ray`].
///
/// This map is cleared and re-populated every frame before any backends run.
/// Ray-based picking backends should use this when possible.
#[derive(Clone, Debug, Default, Resource)]
pub struct RayMap {
    map: HashMap<RayId, Ray>,
}

impl RayMap {
    /// The hash map of all rays cast in the current frame.
    pub fn map(&self) -> &HashMap<RayId, Ray> {
        &self.map
    }

    /// Clears the [`RayMap`] and re-populates it with one ray for each
    /// combination of pointer entity and camera entity where the pointer
    /// intersects the camera's viewport.
    pub fn repopulate(
        mut ray_map: ResMut<Self>,
        primary_window_entity: Query<Entity, With<PrimaryWindow>>,
        cameras: Query<(Entity, &Camera, &GlobalTransform)>,
        pointers: Query<(&PointerId, &PointerLocation)>,
    ) {
        ray_map.map.clear();

        for (camera_entity, camera, camera_tfm) in &cameras {
            if !camera.is_active {
                continue;
            }

            for (&pointer_id, pointer_loc) in &pointers {
                if let Some(ray) = make_ray(&primary_window_entity, camera, camera_tfm, pointer_loc)
                {
                    ray_map
                        .map
                        .insert(RayId::new(camera_entity, pointer_id), ray);
                }
            }
        }
    }
}

fn make_ray(
    primary_window_entity: &Query<Entity, With<PrimaryWindow>>,
    camera: &Camera,
    camera_tfm: &GlobalTransform,
    pointer_loc: &PointerLocation,
) -> Option<Ray> {
    let pointer_loc = pointer_loc.location()?;
    if !pointer_loc.is_in_viewport(camera, primary_window_entity) {
        return None;
    }
    let mut viewport_pos = pointer_loc.position;
    if let Some(viewport) = &camera.viewport {
        let viewport_logical = camera.to_logical(viewport.physical_position)?;
        viewport_pos -= viewport_logical;
    }
    camera.viewport_to_world(camera_tfm, viewport_pos)
}
