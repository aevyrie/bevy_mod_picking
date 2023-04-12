//! A raycasting backend for `bevy_mod_picking` that uses `rapier` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_picking_core::backend::prelude::*;
use bevy_rapier3d::prelude::*;

/// Commonly used imports for the [`bevy_picking_rapier`] crate.
pub mod prelude {
    pub use crate::{RapierBackend, RapierPickSource};
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
pub struct RapierPickSource {
    /// A ray may not exist if the pointer is not active
    pub(crate) ray: Option<Ray>,
}

impl RapierPickSource {
    pub fn ray(&self) -> Option<&Ray> {
        self.ray.as_ref()
    }
}

/// Maps ray casting sources to a [`PointerId`]. A
#[derive(Debug, Clone, Default, Resource)]
pub struct RaycastMap(HashMap<Entity, PointerId>);

/// Updates all picking [`Ray`]s with [`PointerLocation`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(&PointerId, &PointerLocation)>,
    windows: Query<&Window>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut cameras: Query<(Entity, &Camera, &GlobalTransform, &mut RapierPickSource)>,
    mut cast_map: ResMut<RaycastMap>,
) {
    cameras.iter_mut().for_each(|(_, _, _, mut source)| {
        source.ray = None;
    });
    cast_map.0.clear();

    for (pointer_id, pointer_location) in &pointers {
        let pointer_location = match pointer_location.location() {
            Some(l) => l,
            None => continue,
        };
        cameras
            .iter_mut()
            .filter(|(_, camera, _, _)| {
                pointer_location.is_in_viewport(camera, &windows, &primary_window, &images)
            })
            .for_each(|(cam_entity, camera, transform, mut source)| {
                let ray = ray_from_screenspace(pointer_location.position, camera, transform);
                source.ray = ray;
                cast_map.0.insert(cam_entity, *pointer_id);
            });
    }
}

/// Produces [`EntitiesUnderPointer`]s from [`RapierPickRay`] intersections.
fn update_hits(
    rapier_context: Option<Res<RapierContext>>,
    targets: Query<With<RapierPickTarget>>,
    cast_map: Res<RaycastMap>,
    mut sources: Query<(Entity, &Camera, &RapierPickSource)>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    let rapier_context = match rapier_context {
        Some(c) => c,
        None => return,
    };

    todo!("For every camera with a pick source, spawn a pick source for every pointer that uses the same render target as the camera, as children.
    
    entity Camera
        entity pick source for mouse pointer
        entity pick source for touch pointer 1
        etc

    ");

    sources
        .iter()
        .filter_map(|(entity, camera, source)| source.ray.as_ref().map(|ray| (entity, camera, ray)))
        .filter_map(|(entity, camera, ray)| {
            rapier_context
                .cast_ray_and_get_normal(
                    ray.origin,
                    ray.direction,
                    f32::MAX,
                    true,
                    QueryFilter::new().predicate(&|entity| targets.contains(entity)),
                )
                .map(|hit| (entity, camera, hit.0, hit.1.toi, hit.1.point, hit.1.normal))
        })
        .for_each(
            |(cam_entity, camera, hit_entity, depth, position, normal)| {
                let picks = vec![(
                    hit_entity,
                    PickData {
                        depth,
                        position: Some(position),
                        normal: Some(normal),
                    },
                )];
                output.send(EntitiesUnderPointer {
                    pointer: *cast_map.0.get(&cam_entity).unwrap(),
                    picks,
                    order: camera.order,
                });
            },
        );
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
    Some(Ray::new(near, ray_direction))
}

/// A ray used for raycasting
#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Ray {
    /// A point that the ray passes through
    pub origin: Vec3,
    /// A vector that points parallel to the ray
    pub direction: Vec3,
}

impl Ray {
    /// Build a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }
}
