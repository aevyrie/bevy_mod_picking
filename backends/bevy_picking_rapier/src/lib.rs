//! A raycasting backend for `bevy_mod_picking` that uses `rapier` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, window::PrimaryWindow};
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

/// Marks a camera that should be used for rapier raycast picking.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct RapierPickSource;

/// Marks an entity that should be considered for picking raycasts.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct RapierPickTarget;

/// Component to allow pointers to raycast for picking using rapier.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct RapierPickRay {
    /// A ray may not exist if the pointer is not active
    pub ray: Option<Ray>,
}

/// Updates all picking [`Ray`]s with [`PointerLocation`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(Entity, &PointerLocation)>,
    windows: Query<&Window>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
    mut sources: Query<&mut RapierPickRay>,
    cameras: Query<(&Camera, &GlobalTransform), With<RapierPickSource>>,
) {
    sources.iter_mut().for_each(|mut source| {
        source.ray = None;
    });

    for (entity, pointer_location) in &pointers {
        let pointer_location = match pointer_location.location() {
            Some(l) => l,
            None => continue,
        };
        cameras
            .iter()
            .filter(|(camera, _)| {
                pointer_location.is_in_viewport(camera, &windows, &primary_window, &images)
            })
            .for_each(|(camera, transform)| {
                let ray = ray_from_screenspace(pointer_location.position, camera, transform);
                if let Ok(mut source) = sources.get_mut(entity) {
                    source.ray = ray;
                } else {
                    commands.entity(entity).insert(RapierPickRay { ray });
                }
            });
    }
}

/// Produces [`EntitiesUnderPointer`]s from [`RapierPickRay`] intersections.
fn update_hits(
    rapier_context: Option<Res<RapierContext>>,
    sources: Query<(&RapierPickRay, &PointerId)>,
    mut output: EventWriter<EntitiesUnderPointer>,
    targets: Query<With<RapierPickTarget>>,
) {
    let rapier_context = match rapier_context {
        Some(c) => c,
        None => return,
    };

    sources
        .iter()
        .filter_map(|(source, id)| source.ray.as_ref().map(|ray| (id, ray)))
        .filter_map(|(id, ray)| {
            rapier_context
                .cast_ray(
                    ray.origin,
                    ray.direction,
                    f32::MAX,
                    true,
                    QueryFilter::new().predicate(&|entity| targets.contains(entity)),
                )
                .map(|hit| (hit.0, hit.1, id))
        })
        .for_each(|(entity, depth, &id)| {
            let over_list = vec![EntityDepth { entity, depth }];
            output.send(EntitiesUnderPointer {
                pointer: id,
                over_list,
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
