mod focus;
mod highlight;
mod selection;

pub use crate::{
    focus::mesh_focus,
    highlight::{
        get_initial_mesh_button_matl, mesh_highlighting, MeshButtonMaterials, PickableButton,
    },
    selection::{mesh_selection, Selection},
};
use bevy::{prelude::*, ui::FocusPolicy};
pub use bevy_mod_raycast::BoundVol;
use bevy_mod_raycast::*;
use focus::mesh_focus_debug_system;

pub mod pick_stage {
    pub const PICKING: &str = "picking";
}

pub type PickableMesh = RayCastMesh<PickingRaycastSet>;
pub type PickingCamera = RayCastSource<PickingRaycastSet>;

pub struct PickingRaycastSet;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, update_bound_sphere.system())
            .add_system_to_stage(
                stage::POST_UPDATE,
                update_raycast::<PickingRaycastSet>.system(),
            );
    }
}

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, mesh_focus.system())
            .add_system_to_stage(stage::POST_UPDATE, mesh_selection.system());
    }
}

pub struct HighlightablePickingPlugin;
impl Plugin for HighlightablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MeshButtonMaterials>()
            .add_system_to_stage(stage::POST_UPDATE, get_initial_mesh_button_matl.system())
            .add_system_to_stage(stage::POST_UPDATE, mesh_highlighting.system());
    }
}

pub struct DebugPickingPlugin;
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            stage::POST_UPDATE,
            update_debug_cursor::<PickingRaycastSet>.system(),
        )
        .add_system_to_stage(stage::POST_UPDATE, mesh_focus_debug_system.system());
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

#[derive(Bundle, Default)]
pub struct PickableBundle {
    pub pickable_mesh: PickableMesh,
    pub interaction: Interaction,
    pub focus_policy: FocusPolicy,
    pub pickable_button: PickableButton,
    pub selection: Selection,
}
