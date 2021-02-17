mod events;
mod focus;
mod highlight;
mod mouse;
mod selection;

pub use crate::{
    events::{event_debug_system, mesh_events_system, HoverEvent, PickingEvent, SelectionEvent},
    focus::{mesh_focus, Hover},
    highlight::{
        get_initial_mesh_button_material, mesh_highlighting, MeshButtonMaterials, PickableButton,
    },
    mouse::update_pick_source_positions,
    selection::{mesh_selection, Selection},
};
pub use bevy_mod_raycast::BoundVol;

use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_raycast::*;

pub mod pick_stage {
    pub const PICKING: &str = "picking";
}

pub mod pick_labels {
    pub const UPDATE_RAYCAST: &str = "update_raycast";
    pub const MESH_HIGHLIGHTING: &str = "mesh_highlighting";
    pub const MESH_FOCUS: &str = "mesh_focus";
    pub const MESH_EVENTS: &str = "mesh_events";
}

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickableMesh = RayCastMesh<PickingRaycastSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickingCamera = RayCastSource<PickingRaycastSet>;
/// A type alias for the concrete [PluginState](bevy_mod_raycast::PluginState) type used for Picking.
pub type RayCastPluginState = PluginState<PickingRaycastSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and
/// `RayCastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RayCastMesh<PickingRaycastSet>`
/// vs. `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
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
                update_bound_sphere::<PickingRaycastSet>
                    .system()
                    .before(pick_labels::UPDATE_RAYCAST),
            )
            .add_system_to_stage(
                stage::POST_UPDATE,
                update_pick_source_positions
                    .system()
                    .before(pick_labels::UPDATE_RAYCAST),
            )
            .add_system_to_stage(
                stage::POST_UPDATE,
                update_raycast::<PickingRaycastSet>
                    .system()
                    .label(pick_labels::UPDATE_RAYCAST),
            );
    }
}

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<PickingEvent>()
            .add_system_to_stage(
                stage::POST_UPDATE,
                mesh_focus.system().label(pick_labels::MESH_FOCUS),
            )
            .add_system_to_stage(
                stage::POST_UPDATE,
                mesh_selection
                    .system()
                    .before(pick_labels::MESH_EVENTS)
                    .after(pick_labels::MESH_FOCUS),
            )
            .add_system_to_stage(
                stage::POST_UPDATE,
                mesh_events_system.system().label(pick_labels::MESH_EVENTS),
            );
    }
}

pub struct HighlightablePickingPlugin;
impl Plugin for HighlightablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MeshButtonMaterials>()
            .add_system_to_stage(
                stage::POST_UPDATE,
                get_initial_mesh_button_material
                    .system()
                    .before(pick_labels::MESH_HIGHLIGHTING),
            )
            .add_system_to_stage(
                stage::POST_UPDATE,
                mesh_highlighting
                    .system()
                    .label(pick_labels::MESH_HIGHLIGHTING)
                    .after(pick_labels::MESH_EVENTS),
            );
    }
}

pub struct DebugCursorPickingPlugin;
impl Plugin for DebugCursorPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            stage::POST_UPDATE,
            update_debug_cursor::<PickingRaycastSet>.system(),
        );
    }
}

pub struct DebugEventsPickingPlugin;
impl Plugin for DebugEventsPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, event_debug_system.system());
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
            source: PickingCamera::new(),
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
    pub hover: Hover,
}
