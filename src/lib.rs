mod events;
mod focus;
mod highlight;
mod mouse;
mod selection;

pub use crate::{
    events::{event_debug_system, mesh_events_system, HoverEvent, PickingEvent, SelectionEvent},
    focus::{mesh_focus, pause_for_picking_blockers, Hover, PickingBlocker},
    highlight::{
        get_initial_mesh_button_material, mesh_highlighting, MeshButtonMaterials, PickableButton,
    },
    mouse::update_pick_source_positions,
    selection::{mesh_selection, NoDeselect, Selection},
};
pub use bevy_mod_raycast::{BoundVol, Primitive3d, RayCastSource};

use bevy::ecs::schedule::ShouldRun;
use bevy::{prelude::*, ui::FocusPolicy};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickingSystem {
    BuildRays,
    UpdateRaycast,
    Highlighting,
    Selection,
    PauseForBlockers,
    Focus,
    Events,
}

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type PickableMesh = bevy_mod_raycast::RayCastMesh<PickingRaycastSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type PickingCamera = bevy_mod_raycast::RayCastSource<PickingRaycastSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and
/// `RayCastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RayCastMesh<PickingRaycastSet>`
/// vs. `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
pub struct PickingRaycastSet;

pub struct PickingSystemsEnabled(pub bool);

impl Default for PickingSystemsEnabled {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, RunCriteriaLabel)]
pub struct PickingSystemsEnabledCriteria;

fn plugin_enabled(enabled: Res<PickingSystemsEnabled>) -> ShouldRun {
    if enabled.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub struct PausedForBlockers(pub(crate) bool);

impl Default for PausedForBlockers {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub enum UpdatePicks {
    EveryFrame(Vec2),
    OnMouseEvent,
}
impl Default for UpdatePicks {
    fn default() -> Self {
        UpdatePicks::EveryFrame(Vec2::ZERO)
    }
}

pub struct DefaultPickingPlugins;
impl Plugin for DefaultPickingPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugin(PickingPlugin)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin);
    }
}

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingSystemsEnabled>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(plugin_enabled.label(PickingSystemsEnabledCriteria))
                    .with_system(
                        bevy_mod_raycast::update_bound_sphere::<PickingRaycastSet>
                            .before(PickingSystem::UpdateRaycast),
                    )
                    .with_system(update_pick_source_positions.before(PickingSystem::BuildRays))
                    .with_system(
                        bevy_mod_raycast::build_rays::<PickingRaycastSet>
                            .label(PickingSystem::BuildRays)
                            .before(PickingSystem::UpdateRaycast),
                    )
                    .with_system(
                        bevy_mod_raycast::update_raycast::<PickingRaycastSet>
                            .label(PickingSystem::UpdateRaycast),
                    ),
            );
    }
}

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PausedForBlockers>()
            .add_event::<PickingEvent>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(PickingSystemsEnabledCriteria)
                    .with_system(
                        pause_for_picking_blockers
                            .label(PickingSystem::PauseForBlockers)
                            .after(PickingSystem::UpdateRaycast),
                    )
                    .with_system(
                        mesh_focus
                            .label(PickingSystem::Focus)
                            .after(PickingSystem::PauseForBlockers),
                    )
                    .with_system(
                        mesh_selection
                            .label(PickingSystem::Selection)
                            .before(PickingSystem::Events)
                            .after(PickingSystem::Focus),
                    )
                    .with_system(mesh_events_system.label(PickingSystem::Events)),
            );
    }
}

pub struct HighlightablePickingPlugin;
impl Plugin for HighlightablePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshButtonMaterials>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(PickingSystemsEnabledCriteria)
                    .with_system(
                        get_initial_mesh_button_material
                            .after(PickingSystem::UpdateRaycast)
                            .before(PickingSystem::Highlighting),
                    )
                    .with_system(
                        mesh_highlighting
                            .label(PickingSystem::Highlighting)
                            .before(PickingSystem::Events),
                    ),
            );
    }
}

pub struct DebugCursorPickingPlugin;
impl Plugin for DebugCursorPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            bevy_mod_raycast::update_debug_cursor::<PickingRaycastSet>
                .after(PickingSystem::UpdateRaycast),
        );
    }
}

pub struct DebugEventsPickingPlugin;
impl Plugin for DebugEventsPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            event_debug_system
                .with_run_criteria(PickingSystemsEnabledCriteria)
                .after(PickingSystem::Events),
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
