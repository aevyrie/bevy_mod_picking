use bevy::prelude::*;
use bevy_mod_raycast::{Ray3d, RayCastSource};
use bevy_picking_core::{
    picking::{cursor::CursorInput, hit::CursorHit, CorePickingSystem},
    simple_criteria, PickingSettings,
};

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickingTarget = bevy_mod_raycast::RayCastMesh<RaycastPickingSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickingSource = RayCastSource<RaycastPickingSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and
/// `RayCastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RayCastMesh<RaycastPickingSet>`
/// vs. `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
pub struct RaycastPickingSet;

pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .with_run_criteria(|state: Res<PickingSettings>| {
                    simple_criteria(state.enable_picking)
                })
                .with_system(
                    build_rays_from_cursors
                        .label(RaycastSystem::UpdateSourceRays)
                        .before(RaycastSystem::UpdateRaycast),
                )
                .with_system(
                    bevy_mod_raycast::update_raycast::<RaycastPickingSet>
                        .label(RaycastSystem::UpdateRaycast)
                        .before(RaycastSystem::UpdateIntersections),
                )
                .with_system(
                    update_hits
                        .label(RaycastSystem::UpdateIntersections)
                        .before(CorePickingSystem::Focus)
                        .before(CorePickingSystem::InitialHighlights),
                ),
        );
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, SystemLabel)]
pub enum RaycastSystem {
    UpdateSourceRays,
    UpdateRaycast,
    UpdateIntersections,
}

/// Update Screenspace ray cast sources with the current cursor positions
pub fn build_rays_from_cursors(
    mut commands: Commands,
    mut sources: Query<&mut PickingSource>,
    cursors: Query<(Entity, &CursorInput), Changed<CursorInput>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    for (entity, cursor) in cursors.iter() {
        cameras
            .iter()
            .filter(|(camera, _)| cursor.is_same_target(camera))
            .filter(|(camera, _)| cursor.is_in_viewport(camera))
            .map(|(camera, transform)| Ray3d::from_screenspace(cursor.position, camera, transform))
            .for_each(|ray| update_raycast_source(&mut sources, entity, ray, &mut commands));
    }
}

#[inline]
fn update_raycast_source(
    sources: &mut Query<&mut PickingSource>,
    entity: Entity,
    ray: Option<Ray3d>,
    commands: &mut Commands,
) {
    if let Ok(mut source) = sources.get_mut(entity) {
        source.ray = ray;
    } else {
        let mut source = PickingSource::default();
        source.ray = ray;
        commands.entity(entity).insert(source);
    }
}

fn update_hits(mut sources: Query<(&PickingSource, &mut CursorHit, &CursorInput)>) {
    for (source, mut cursor_hit, cursor) in sources.iter_mut() {
        if !cursor.enabled || source.intersect_top().is_none() {
            cursor_hit.entities.clear();
        } else if let Some((entity, _)) = source.intersect_top() {
            cursor_hit.entities = vec![entity];
        };
    }
}
