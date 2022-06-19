use bevy::prelude::*;
use bevy_mod_raycast::{Ray3d, RayCastSource};
use bevy_picking_core::{
    picking::{cursor::Cursor, CoreSystem},
    simple_criteria, PickingSettings,
};

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type RaycastTarget = bevy_mod_raycast::RayCastMesh<RaycastPickingSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type RaycastSource = RayCastSource<RaycastPickingSet>;

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
                    bevy_mod_raycast::update_intersections::<RaycastPickingSet>
                        .label(RaycastSystem::UpdateIntersections)
                        .before(CoreSystem::PauseForBlockers)
                        .before(CoreSystem::InitialHighlights),
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
    mut sources: Query<&mut RaycastSource>,
    cursors: Query<(Entity, &Cursor)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    for (entity, cursor) in cursors.iter() {
        cameras
            .iter()
            .filter(|(camera, _)| is_same_render_target(camera, cursor))
            .for_each(|(camera, transform)| {
                let ray = is_cursor_in_viewport(camera, cursor)
                    .then(|| Ray3d::from_screenspace(cursor.position, camera, transform))
                    .flatten();
                update_raycast_source(&mut sources, entity, ray, &mut commands);
            });
    }
}

#[inline]
fn update_raycast_source(
    sources: &mut Query<&mut RaycastSource>,
    entity: Entity,
    ray: Option<Ray3d>,
    commands: &mut Commands,
) {
    if let Ok(mut source) = sources.get_mut(entity) {
        if source.ray != ray {
            source.ray = ray;
            info!("Cursor updated.");
        }
    } else {
        let mut source = RaycastSource::default();
        if source.ray != ray {
            source.ray = ray;
            info!("Cursor created.");
        }
        commands.entity(entity).insert(source);
    }
}

#[inline]
fn is_cursor_in_viewport(camera: &Camera, cursor: &Cursor) -> bool {
    camera
        .logical_viewport_rect()
        .map(|(min, max)| {
            (cursor.position - min).min_element() >= 0.0
                && (cursor.position - max).max_element() <= 0.0
        })
        .unwrap_or(false)
}

#[inline]
fn is_same_render_target(camera: &Camera, cursor: &Cursor) -> bool {
    camera.target == cursor.target
}
