use bevy::prelude::*;
use bevy_mod_raycast::{Ray3d, RayCastSource};
use bevy_picking_core::{
    backend::{EntitiesUnderPointer, PointerOverMetadata},
    input::PointerPosition,
    PickStage, PickingSettings, PointerId,
};

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickRaycastTarget = bevy_mod_raycast::RayCastMesh<RaycastPickingSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickRaycastSource = RayCastSource<RaycastPickingSet>;

/// This unit struct is used to tag the generic ray casting types
/// [RayCastMesh](bevy_mod_raycast::RayCastMesh) and [`RayCastSource`].
pub struct RaycastPickingSet;

pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .label(PickStage::Backend)
                .after(PickStage::Input)
                .before(PickStage::Events)
                .with_run_criteria(|state: Res<PickingSettings>| state.backend)
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

/// Builds rays and updates raycasting [`PickRaycastSource`]s from [`PointerPosition`]s.
pub fn build_rays_from_pointers(
    pointers: Query<(Entity, &PointerId, &PointerPosition)>,
    mut commands: Commands,
    mut sources: Query<&mut PickRaycastSource>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    sources.iter_mut().for_each(|mut source| {
        source.ray = None;
        source.intersections_mut().clear()
    });

    for (entity, _id, location) in pointers.iter() {
        let location = if let Some(location) = location.location() {
            location
        } else {
            continue;
        };
        cameras
            .iter()
            .filter(|(camera, _)| location.is_same_target(camera))
            .filter(|(camera, _)| location.is_in_viewport(camera))
            .map(|(camera, transform)| {
                Ray3d::from_screenspace(location.position, camera, transform)
            })
            .for_each(|ray| {
                if let Ok(mut source) = sources.get_mut(entity) {
                    source.ray = ray;
                } else {
                    let mut source = PickRaycastSource::default();
                    source.ray = ray;
                    commands.entity(entity).insert(source);
                }
            });
    }
}

/// Produces [`EntitiesUnderPointer`]s from [`PickingSource`] intersections.
fn update_hits(
    mut sources: Query<(&PickRaycastSource, &PointerId)>,
    mut output: EventWriter<EntitiesUnderPointer>,
) {
    for (source, &id) in sources.iter_mut() {
        let over: Vec<PointerOverMetadata> = source
            .intersect_list()
            .iter()
            .flat_map(|inner| {
                inner
                    .iter()
                    .map(|(entity, intersection)| PointerOverMetadata {
                        entity: *entity,
                        depth: intersection.distance(),
                    })
            })
            .collect();

        if !over.is_empty() {
            output.send(EntitiesUnderPointer {
                id,
                over_list: over,
            });
        }
    }
}
