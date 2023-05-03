//! A raycasting backend for `bevy_mod_picking` that uses `rapier` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_picking_core::backend::prelude::*;
use bevy_rapier3d::prelude::*;

/// Commonly used imports.
pub mod prelude {
    pub use crate::{RapierBackend, RapierPickCamera};
}

/// Adds the `rapier` raycasting picking backend to your app.
#[derive(Clone)]
pub struct RapierBackend;
impl PickingBackend for RapierBackend {}
impl Plugin for RapierBackend {
    fn build(&self, app: &mut App) {
        app.add_system(build_rays_from_pointers.in_set(PickSet::PostInput))
            .add_systems((update_hits,).chain().in_set(PickSet::Backend));
    }
}

/// Marks an entity that should be considered for picking raycasts.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct RapierPickTarget;

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
    windows: Query<&Window>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
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
                pointer_location.is_in_viewport(camera, &windows, &primary_window, &images)
            })
            .for_each(|(camera, transform, mut source)| {
                let pointer_pos = pointer_location.position;
                if let Some(ray) = ray_from_screenspace(pointer_pos, camera, transform) {
                    source.ray_map.insert(*pointer_id, ray);
                }
            });
    }
}

/// Produces [`PointerHits`]s from [`RapierPickRay`] intersections.
fn update_hits(
    rapier_context: Option<Res<RapierContext>>,
    targets: Query<(With<RapierPickTarget>, With<Pickable>)>,
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
                    QueryFilter::new().predicate(&|entity| targets.contains(entity)),
                )
                .map(|(target, intersection)| {
                    (cam_entity, cam_order, pointer, target, intersection)
                })
        })
        .for_each(|(cam_entity, cam_order, pointer, target, intersection)| {
            let hit = HitData {
                camera: cam_entity,
                depth: intersection.toi,
                position: Some(intersection.point),
                normal: Some(intersection.normal),
            };
            output.send(PointerHits {
                pointer: pointer,
                picks: vec![(target, hit)],
                order: cam_order,
            });
        });
}

/// Create a [`Ray`] from a camera's screenspace coordinates.
pub fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray> {
    let view = camera_transform.compute_matrix();
    let screen_size = camera.logical_target_size()?;
    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z * 1000.0).z;
    let near_ndc = projection.project_point3(Vec3::NEG_Z * 0.001).z;
    let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;
    Some(Ray {
        origin: near,
        direction: ray_direction,
    })
}
