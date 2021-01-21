mod highlight;
mod interactable;
mod select;

pub use crate::{
    highlight::{HighlightablePickMesh, PickHighlightParams},
    interactable::{HoverEvents, InteractableMesh, InteractablePickingPlugin, MouseDownEvents},
    select::SelectablePickMesh,
};

use bevy::prelude::*;
use bevy_mod_raycast::*;

pub mod pick_stage {
    pub const PICKING: &str = "picking";
    pub const POST_PICKING: &str = "post_picking";
}

pub struct PickingRaycastSet;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickHighlightParams>()
            .add_startup_system(setup_debug_cursor::<PickingRaycastSet>.system())
            .add_stage_after(
                stage::POST_UPDATE,
                pick_stage::PICKING,
                SystemStage::serial(),
            )
            .add_stage_after(
                pick_stage::PICKING,
                pick_stage::POST_PICKING,
                SystemStage::serial(),
            )
            .add_system_to_stage(stage::POST_UPDATE, build_bound_sphere.system())
            .add_system_to_stage(
                pick_stage::PICKING,
                update_raycast::<PickingRaycastSet>.system(),
            )
            .add_system_to_stage(
                pick_stage::POST_PICKING,
                update_debug_cursor::<PickingRaycastSet>.system(),
            )
            .add_plugin(InteractablePickingPlugin);
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

pub fn picking_camera() -> RayCastSource<PickingRaycastSet> {
    RayCastSource::<PickingRaycastSet>::new(RayCastMethod::CameraCursor(
        UpdateOn::EveryFrame(Vec2::zero()),
        EventReader::default(),
    ))
}

/*
pub type PickableMesh = RayCastMesh<PickingRaycastSet>;

pub struct PickingCamera(RayCastSource<PickingRaycastSet>);
impl Deref for PickingCamera {
    type Target = RayCastSource<PickingRaycastSet>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for PickingCamera {
    fn default() -> Self {
        PickingCamera(RayCastSource::<PickingRaycastSet>::new(
            RayCastMethod::CameraCursor(UpdateOn::EveryFrame(Vec2::zero()), EventReader::default()),
        ))
    }
}
*/
