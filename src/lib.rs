mod focus;
mod highlight;
mod mouse;
mod selection;

pub use crate::{
    focus::mesh_focus,
    highlight::{
        get_initial_mesh_button_matl, mesh_highlighting, MeshButtonMaterials, PickableButton,
    },
    mouse::update_pick_source_positions,
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
pub type RayCastPluginState = PluginState<PickingRaycastSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and `RayCastSource`.
/// This means that all Picking ray casts are of the same type. Consequently, any meshes or ray
/// sources that are being used by the picking plugin can be used by other ray casting systems
/// because they will have distinct types, e.g.: `RayCastMesh<PickingRaycastSet>` vs.
/// `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
#[derive(Default)]
pub struct PickingRaycastSet;

pub struct PickingPluginState {
    pub enabled: bool,
    paused_for_ui: bool,
}
impl Default for PickingPluginState {
    fn default() -> Self {
        PickingPluginState {
            enabled: true,
            paused_for_ui: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatePicks {
    EveryFrame(Vec2),
    OnMouseEvent,
}
impl Default for UpdatePicks {
    fn default() -> Self {
        UpdatePicks::EveryFrame(Vec2::zero())
    }
}

fn update_state(
    mut raycast_state: ResMut<RayCastPluginState>,
    picking_state: Res<PickingPluginState>,
) {
    raycast_state.enabled = picking_state.enabled;
}

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<RayCastPluginState>()
            .init_resource::<PickingPluginState>()
            .add_system_to_stage(stage::POST_UPDATE, update_state.system())
            .add_system_to_stage(
                stage::POST_UPDATE,
                update_bound_sphere::<PickingRaycastSet>.system(),
            )
            .add_system_to_stage(stage::POST_UPDATE, update_pick_source_positions.system())
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
    pub update: UpdatePicks,
}

impl Default for PickingCameraBundle {
    fn default() -> Self {
        PickingCameraBundle {
            source: PickingCamera::new(RayCastMethod::Screenspace(Vec2::zero())),
            update: UpdatePicks::default(),
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
