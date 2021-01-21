mod highlight;
mod interactable;
mod select;

pub use crate::{
    highlight::{HighlightablePickMesh, PickHighlightParams},
    interactable::{HoverEvents, InteractableMesh, InteractablePickingPlugin, MouseDownEvents},
    select::SelectablePickMesh,
};

use bevy::prelude::*;
use bevy_photon::*;

pub struct PickingRaycastSet;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickHighlightParams>()
            .add_plugin(InteractablePickingPlugin)
            .add_startup_system(setup_debug_cursor::<PickingRaycastSet>.system())
            .add_system(build_bound_sphere.system())
            .add_stage_after(stage::POST_UPDATE, "picking", SystemStage::serial())
            .add_system_to_stage("picking", update_raycast::<PickingRaycastSet>.system())
            .add_stage_after("picking", "post_picking", SystemStage::serial())
            .add_system_to_stage(
                "post_picking",
                update_debug_cursor::<PickingRaycastSet>.system(),
            );
    }
}

pub struct DebugPickingPlugin;
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update_debug_cursor::<PickingRaycastSet>.system())
            .add_startup_system(setup_debug_cursor::<PickingRaycastSet>.system());
    }
}

#[derive(Bundle, Default)]
pub struct PickableBundle {
    mesh: RayCastMesh<PickingRaycastSet>,
    interact: InteractableMesh,
    highlight: HighlightablePickMesh,
    select: SelectablePickMesh,
}

pub fn pickable_mesh() -> RayCastMesh<PickingRaycastSet> {
    RayCastMesh::<PickingRaycastSet>::default()
}

pub fn picking_camera() -> RayCastSource<PickingRaycastSet> {
    RayCastSource::<PickingRaycastSet>::new(RayCastMethod::CameraCursor(
        UpdateOn::EveryFrame(Vec2::zero()),
        EventReader::default(),
    ))
}

