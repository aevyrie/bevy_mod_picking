use bevy::prelude::*;
use bevy_mod_raycast::{Ray3d, RayCastSource};
use bevy_picking_core::{
    backend::CursorOver, input::CursorLocation, simple_criteria, PickStage, PickingSettings,
};

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickingTarget = bevy_mod_raycast::RayCastMesh<RaycastPickingSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickingSource = RayCastSource<RaycastPickingSet>;

/// This unit struct is used to tag the generic ray casting types
/// [RayCastMesh](bevy_mod_raycast::RayCastMesh) and [`RayCastSource`]. This means that all Picking
/// ray casts are of the same type. Consequently, any meshes or ray sources that are being used by
/// the picking plugin can be used by other ray casting systems because they will have distinct
/// types, e.g.: `RayCastMesh<RaycastPickingSet>` vs. `RayCastMesh<MySuperCoolRaycastingType>`, and
/// as such wil not result in collisions.
pub struct RaycastPickingSet;

pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .label(PickStage::Backend)
                .after(PickStage::Input)
                .before(PickStage::Focus)
                .with_run_criteria(|state: Res<PickingSettings>| {
                    simple_criteria(state.enable_backend)
                })
                .with_system(
                    build_rays_from_cursors
                        .before(bevy_mod_raycast::update_raycast::<RaycastPickingSet>),
                )
                .with_system(bevy_mod_raycast::update_raycast::<RaycastPickingSet>)
                .with_system(
                    update_hits.after(bevy_mod_raycast::update_raycast::<RaycastPickingSet>),
                ),
        );
    }
}

/// Builds rays and updates raycasting [`PickingSource`]s from [`CursorLocation`]s.
pub fn build_rays_from_cursors(
    mut commands: Commands,
    mut sources: Query<&mut PickingSource>,
    cursors: Query<(Entity, &CursorLocation), Changed<CursorLocation>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    for (entity, cursor) in cursors.iter() {
        if let Some(loc) = &cursor.location {
            cameras
                .iter()
                .filter(|(camera, _)| cursor.is_same_target(camera))
                .filter(|(camera, _)| cursor.is_in_viewport(camera))
                .map(|(camera, transform)| Ray3d::from_screenspace(loc.position, camera, transform))
                .for_each(|ray| update_raycast_source(&mut sources, entity, ray, &mut commands));
        } else {
            update_raycast_source(&mut sources, entity, None, &mut commands);
        }
    }
}

/// Raycasting sources are added to cursor entities
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

fn update_hits(mut sources: Query<(&PickingSource, &mut CursorOver)>) {
    for (source, mut cursor_over) in sources.iter_mut() {
        // because the raycasting plugin doesn't update when the ray is `None`, we need to check for
        // that case here.
        if source.ray.is_none() || source.intersect_top().is_none() {
            if !cursor_over.as_ref().entities().is_empty() {
                cursor_over.clear();
            }
        } else {
            let new_list: Vec<Entity> = source
                .intersect_list()
                .iter()
                .flat_map(|inner| inner.iter().map(|(entity, _)| *entity))
                .collect();
            if !new_list.is_empty() && new_list != cursor_over.entities() {
                cursor_over.entities = new_list;
            }
        };
    }
}
