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
    selection::{mesh_selection, NoDeselect, Selection},
};
pub use bevy_mod_raycast::{BoundVol, Primitive3d, RayCastSource};

use bevy::{prelude::*, ui::FocusPolicy};

pub mod pick_stage {
    pub const PICKING: &str = "picking";
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickingSystem {
    BuildRays,
    UpdateRaycast,
    Highlighting,
    Selection,
    Focus,
    Events,
}

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickableMesh = bevy_mod_raycast::RayCastMesh<PickingRaycastSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickingCamera = bevy_mod_raycast::RayCastSource<PickingRaycastSet>;
/// A type alias for the concrete [PluginState](bevy_mod_raycast::PluginState) type used for Picking.
pub type RayCastPluginState = bevy_mod_raycast::PluginState<PickingRaycastSet>;

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
        UpdatePicks::EveryFrame(Vec2::ZERO)
    }
}

fn update_state(
    mut raycast_state: ResMut<RayCastPluginState>,
    picking_state: Res<PickingPluginState>,
) {
    raycast_state.enabled = picking_state.enabled;
}

pub struct DefaultPickingPlugins;
impl Plugin for DefaultPickingPlugins {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin);
    }
}

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<RayCastPluginState>()
            .init_resource::<PickingPluginState>()
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                bevy_mod_raycast::update_bound_sphere::<PickingRaycastSet>
                    .system()
                    .before(PickingSystem::UpdateRaycast),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_pick_source_positions
                    .system()
                    .before(PickingSystem::BuildRays),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                bevy_mod_raycast::build_rays::<PickingRaycastSet>
                    .system()
                    .label(PickingSystem::BuildRays)
                    .before(PickingSystem::UpdateRaycast),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                bevy_mod_raycast::update_raycast::<PickingRaycastSet>
                    .system()
                    .label(PickingSystem::UpdateRaycast),
            );
    }
}

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<PickingEvent>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                mesh_focus
                    .system()
                    .label(PickingSystem::Focus)
                    .after(PickingSystem::UpdateRaycast),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                mesh_selection
                    .system()
                    .label(PickingSystem::Selection)
                    .before(PickingSystem::Events)
                    .after(PickingSystem::Focus),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                mesh_events_system.system().label(PickingSystem::Events),
            );
    }
}

pub struct HighlightablePickingPlugin;
impl Plugin for HighlightablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MeshButtonMaterials>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                get_initial_mesh_button_material
                    .system()
                    .after(PickingSystem::UpdateRaycast)
                    .before(PickingSystem::Highlighting),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                mesh_highlighting
                    .system()
                    .label(PickingSystem::Highlighting)
                    .before(PickingSystem::Events),
            );
    }
}

pub struct DebugCursorPickingPlugin;
impl Plugin for DebugCursorPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            bevy_mod_raycast::update_debug_cursor::<PickingRaycastSet>
                .system()
                .after(PickingSystem::UpdateRaycast),
        );
    }
}

pub struct DebugEventsPickingPlugin;
impl Plugin for DebugEventsPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            event_debug_system.system().after(PickingSystem::Events),
        );
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
