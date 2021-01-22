mod highlight;
mod interactable;
mod select;

pub use crate::{
    highlight::{pick_highlighting, HighlightablePickMesh, PickHighlightParams},
    interactable::{
        generate_click_events, generate_hover_events, HoverEvents, InteractableMesh,
        MouseButtonEvents,
    },
    select::{select_mesh, SelectablePickMesh},
};
use bevy::prelude::*;
use bevy_mod_raycast::*;
pub use bevy_mod_raycast::BoundVol;

pub mod pick_stage {
    pub const PICKING: &str = "picking";
    pub const POST_PICKING: &str = "post_picking";
}

pub type PickableMesh = RayCastMesh<PickingRaycastSet>;
pub type PickingCamera = RayCastSource<PickingRaycastSet>;

pub struct PickingRaycastSet;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickHighlightParams>()
            .add_stage_after(
                stage::POST_UPDATE,
                pick_stage::PICKING,
                SystemStage::parallel(),
            )
            .add_stage_after(
                pick_stage::PICKING,
                pick_stage::POST_PICKING,
                SystemStage::parallel(),
            )
            .add_system_to_stage(stage::POST_UPDATE, build_new_bound_sphere.system())
            .add_system_to_stage(stage::POST_UPDATE, update_bound_sphere_changed_mesh.system())
            .add_system_to_stage(
                pick_stage::PICKING,
                update_raycast::<PickingRaycastSet>.system(),
            );
    }
}

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(pick_stage::POST_PICKING, generate_hover_events.system())
            .add_system_to_stage(pick_stage::POST_PICKING, generate_click_events.system())
            .add_system_to_stage(pick_stage::POST_PICKING, select_mesh.system())
            .add_system_to_stage(pick_stage::POST_PICKING, pick_highlighting.system());
    }
}

pub struct DebugPickingPlugin;
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            pick_stage::POST_PICKING,
            update_debug_cursor::<PickingRaycastSet>.system(),
        );
    }
}

#[derive(Bundle)]
pub struct PickingCameraBundle {
    pub source: PickingCamera,
}

impl Default for PickingCameraBundle {
    fn default() -> Self {
        PickingCameraBundle {
            source: PickingCamera::new(RayCastMethod::CameraCursor(
                UpdateOn::EveryFrame(Vec2::zero()),
                EventReader::default(),
            )),
        }
    }
}

#[derive(Bundle)]
pub struct PickableBundle {
    pub mesh: PickableMesh,
    pub interact: InteractableMesh,
    pub highlight: HighlightablePickMesh,
    pub select: SelectablePickMesh,
}

impl Default for PickableBundle {
    fn default() -> Self {
        PickableBundle {
            mesh: PickableMesh::default(),
            interact: InteractableMesh::default(),
            highlight: HighlightablePickMesh::default(),
            select: SelectablePickMesh::default(),
        }
    }
}
