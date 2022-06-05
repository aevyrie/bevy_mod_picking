use bevy::{prelude::*, utils::HashMap};
use bevy_mod_raycast::Ray3d;
use bevy_picking_core::{
    picking::{cursor::Cursor, CoreSystem},
    simple_criteria, PickingSettings,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, SystemLabel)]
pub enum RaycastSystem {
    UpdateSourceRays,
    UpdateRaycast,
    UpdateIntersections,
}

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type RaycastTarget = bevy_mod_raycast::RayCastMesh<RaycastPickingSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type RaycastSource = bevy_mod_raycast::RayCastSource<RaycastPickingSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and
/// `RayCastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RayCastMesh<RaycastPickingSet>`
/// vs. `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
pub struct RaycastPickingSet;

pub struct RaycastPickingPlugin;
impl Plugin for RaycastPickingPlugin {
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
                    bevy_mod_raycast::update_intersections::<RaycastPickingSet>
                        .label(RaycastSystem::UpdateIntersections)
                        .before(CoreSystem::PauseForBlockers)
                        .before(CoreSystem::InitialHighlights),
                ),
        );
    }
}

pub struct CursorMap(HashMap<u64, Entity>);

/// Update Screenspace ray cast sources with the current cursor positions
pub fn build_rays_from_cursors(
    mut commands: Commands,
    mut sources: Query<&mut RaycastSource>,
    cursors: Query<(Entity, &Cursor)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    for (entity, cursor) in cursors.iter() {
        for (camera, transform) in cameras.iter() {
            //TODO: check if cursor is outside of viewport and return early
            if camera.target == cursor.target {
                let ray = Ray3d::from_screenspace(cursor.position, camera, transform);
                if let Ok(mut source) = sources.get_mut(entity) {
                    source.ray = ray;
                } else {
                    let mut source = RaycastSource::default();
                    source.ray = ray;
                    commands.entity(entity).insert(source);
                }
            }
        }
    }
}
