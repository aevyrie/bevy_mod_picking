use bevy::prelude::*;

pub enum RaycastSystem {
    UpdatePickSourcePositions,
    BuildRays,
    UpdateRaycast,
    UpdateIntersections,
}

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickableTarget = bevy_mod_raycast::RayCastMesh<PickingRaycastSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickingSource = bevy_mod_raycast::RayCastSource<PickingRaycastSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and
/// `RayCastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RayCastMesh<PickingRaycastSet>`
/// vs. `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
pub struct PickingRaycastSet;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .with_run_criteria(|state: Res<PickingPluginsState>| {
                    simple_criteria(state.enable_picking)
                })
                .with_system(
                    bevy_mod_raycast::build_rays::<PickingRaycastSet>
                        .label(RaycastSystem::BuildRays)
                        .after(RaycastSystem::UpdatePickSourcePositions)
                        .before(PickingSystem::UpdateRaycast),
                )
                .with_system(
                    bevy_mod_raycast::update_raycast::<PickingRaycastSet>
                        .label(RaycastSystem::UpdateRaycast)
                        .before(RaycastSystem::UpdateIntersections),
                )
                .with_system(
                    bevy_mod_raycast::update_intersections::<PickingRaycastSet>
                        .label(RaycastSystem::UpdateIntersections)
                        .before(PickingSystem::PauseForBlockers)
                        .before(PickingSystem::InitialHighlights),
                )
                .with_system(
                    update_pick_source_positions.label(PickingSystem::UpdatePickSourcePositions),
                ),
        );
    }
}
