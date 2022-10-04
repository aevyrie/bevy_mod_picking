//! A raycasting backend for `bevy_mod_picking` that uses `bevy_mod_raycast` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_mod_raycast::{Ray3d, RayCastSource};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_raycast`](crate) crate.
pub mod prelude {
    pub use crate::{PickRaycastSource, PickRaycastTarget, RaycastPlugin};
}

/// Adds the raycasting picking backend to your app.
pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .label(PickStage::Backend)
                .with_system(build_rays_from_pointers)
                .with_system(
                    bevy_mod_raycast::update_raycast::<RaycastPickingSet>
                        .after(build_rays_from_pointers)
                        .before(update_hits),
                )
                .with_system(update_hits),
        );
    }
}

/// Marks an entity that should be pickable with [`bevy_mod_raycast`] ray casts.
pub type PickRaycastTarget = bevy_mod_raycast::RayCastMesh<RaycastPickingSet>;

/// Marks a camera that should be used for [`bevy_mod_raycast`] picking.
#[derive(Debug, Default, Clone, Component)]
pub struct PickRaycastSource;

/// This unit struct is used to tag the generic ray casting types
/// [`RayCastMesh`](bevy_mod_raycast::RayCastMesh) and [`RayCastSource`].
pub struct RaycastPickingSet;

/// Builds rays and updates raycasting [`PickRaycastSource`]s from [`PointerLocation`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(Entity, &PointerLocation)>,
    mut commands: Commands,
    mut sources: Query<&mut RayCastSource<RaycastPickingSet>>,
    cameras: Query<(&Camera, &GlobalTransform), With<PickRaycastSource>>,
) {
    sources.iter_mut().for_each(|mut source| {
        source.ray = None;
        source.intersections_mut().clear()
    });

    for (entity, pointer_location) in &pointers {
        let pointer_location = match pointer_location.location() {
            Some(l) => l,
            None => continue,
        };
        cameras
            .iter()
            .filter(|(camera, _)| pointer_location.is_in_viewport(camera))
            .for_each(|(camera, transform)| {
                let ray = Ray3d::from_screenspace(pointer_location.position, camera, transform);
                if let Ok(mut source) = sources.get_mut(entity) {
                    source.ray = ray;
                } else {
                    let mut source = RayCastSource::<RaycastPickingSet>::default();
                    source.ray = ray;
                    commands.entity(entity).insert(source);
                }
            });
    }
}

/// Produces [`EntitiesUnderPointer`]s from [`PickRaycastSource`] intersections.
fn update_hits(
    mut sources: Query<(&RayCastSource<RaycastPickingSet>, &PointerId)>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    for (source, &id) in &mut sources {
        let under_cursor: Vec<EntityDepth> = source
            .intersect_list()
            .iter()
            .flat_map(|inner| {
                inner.iter().map(|(entity, intersection)| EntityDepth {
                    entity: *entity,
                    depth: intersection.distance(),
                })
            })
            .collect();

        if !under_cursor.is_empty() {
            output.send(EntitiesUnderPointer {
                pointer: id,
                over_list: under_cursor,
            });
        }
    }
}
