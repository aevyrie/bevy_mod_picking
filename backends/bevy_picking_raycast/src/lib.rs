//! A raycasting backend for `bevy_mod_picking` that uses `bevy_mod_raycast` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_mod_raycast::{Ray3d, RaycastSource};
use bevy_picking_core::backend::{prelude::*, PickingBackend};

/// Commonly used imports for the [`bevy_picking_raycast`](crate) crate.
pub mod prelude {
    pub use crate::{PickRaycastCamera, PickRaycastTarget, RaycastBackend};
}

/// Adds the raycasting picking backend to your app.
#[derive(Clone)]
pub struct RaycastBackend;
impl PickingBackend for RaycastBackend {}
impl Plugin for RaycastBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (build_rays_from_pointers, spawn_raycast_sources)
                .chain()
                .in_set(PickSet::PostInput),
        )
        .add_systems(
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
/// [`RaycastMesh`](bevy_mod_raycast::RaycastMesh) and [`RaycastSource`].
#[derive(Reflect, Clone)]
pub struct RaycastPickingSet;

/// Marks an entity that should be pickable with [`bevy_mod_raycast`] ray casts.
pub type PickRaycastTarget = bevy_mod_raycast::RaycastMesh<RaycastPickingSet>;

/// Marks a camera that should be used for picking with [`bevy_mod_raycast`].
#[derive(Debug, Default, Clone, Component, Reflect)]
pub struct PickRaycastCamera {
    #[reflect(ignore)]
    /// Maps the pointers visible to this [`PickRaycastSource`] to their corresponding ray. We need
    /// to create a map because many pointers may be visible to this camera.
    ray_map: HashMap<PointerId, Ray3d>,
}

// --
//
// TODO:
//
// The following design, where we need to add children to the cameras, only exists because
// `bevy_mod_raycast` only supports raycasting via components. Ideally, we would be able to run
// raycasts on demand without needing to supply them as components on entities.
//
// --

/// Builds rays and updates raycasting [`PickRaycastCamera`]s from [`PointerLocation`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(&PointerId, &PointerLocation)>,
    windows: Query<&Window>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut picking_cameras: Query<(&Camera, &GlobalTransform, &mut PickRaycastCamera)>,
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
                if let Some(ray) = Ray3d::from_screenspace(pointer_pos, camera, transform) {
                    source.ray_map.insert(*pointer_id, ray);
                }
            });
    }
}

/// A newtype, used solely to mark the [`RaycastSource`] children on the [`PickRaycastCamera`] so we
/// know what pointer they are associated with.
#[derive(Component)]
struct PointerMarker(PointerId);

/// Using the rays in each [`PickRaycastCamera`], updates their child [`RaycastSource`]s.
pub fn spawn_raycast_sources(
    mut commands: Commands,
    picking_cameras: Query<(Entity, &PickRaycastCamera)>,
    child_sources: Query<Entity, With<RaycastSource<RaycastPickingSet>>>,
) {
    child_sources
        .iter()
        .for_each(|pick_source| commands.entity(pick_source).despawn_recursive());

    picking_cameras.iter().for_each(|(entity, pick_cam)| {
        pick_cam.ray_map.iter().for_each(|(pointer, ray)| {
            let mut new_source = RaycastSource::<RaycastPickingSet>::default();
            new_source.ray = Some(*ray);
            let pointer_marker = PointerMarker(*pointer);
            let new_child = commands.spawn((new_source, pointer_marker)).id();
            commands.entity(entity).add_child(new_child);
        })
    })
}

/// Produces [`EntitiesUnderPointer`]s from [`RaycastSource`] intersections.
fn update_hits(
    pick_cameras: Query<(Entity, &Camera), With<PickRaycastCamera>>,
    mut pick_sources: Query<(&PointerMarker, &RaycastSource<RaycastPickingSet>, &Parent)>,
    mut output_events: EventWriter<EntitiesUnderPointer>,
) {
    pick_sources
        .iter_mut()
        .filter_map(|(pointer, pick_source, parent)| {
            pick_cameras
                .get(parent.get())
                .map(|(entity, camera)| (pointer, pick_source, entity, camera))
                .ok()
        })
        .for_each(|(pointer_marker, pick_source, cam_entity, camera)| {
            let under_cursor: Vec<(Entity, PickData)> = pick_source
                .intersections()
                .iter()
                .map(|(entity, intersection)| {
                    (
                        *entity,
                        PickData {
                            camera: cam_entity,
                            depth: intersection.distance(),
                            position: Some(intersection.position()),
                            normal: Some(intersection.normal()),
                        },
                    )
                })
                .collect();

            if !under_cursor.is_empty() {
                output_events.send(EntitiesUnderPointer {
                    pointer: pointer_marker.0,
                    picks: under_cursor,
                    order: camera.order,
                });
            }
        });
}
