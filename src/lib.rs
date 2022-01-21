mod events;
mod focus;
mod highlight;
mod mouse;
mod selection;

use std::marker::PhantomData;

pub use crate::{
    events::{event_debug_system, mesh_events_system, HoverEvent, PickingEvent, SelectionEvent},
    focus::{mesh_focus, pause_for_picking_blockers, Hover, PickingBlocker},
    highlight::{
        get_initial_mesh_button_material, mesh_highlighting, FromWorldHelper, MeshButtonMaterials,
        PickableButton, StandardMaterialPickingColors,
    },
    mouse::update_pick_source_positions,
    selection::{mesh_selection, NoDeselect, Selection},
};
pub use bevy_mod_raycast::{Primitive3d, RayCastSource};

use bevy::{
    app::PluginGroupBuilder, asset::Asset, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy,
};

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

pub struct PickingPluginsState {
    pub enable_picking: bool,
    pub enable_highlighting: bool,
    pub enable_interacting: bool,
    pub update_debug_cursor: bool,
    pub print_debug_events: bool,
}

impl Default for PickingPluginsState {
    fn default() -> Self {
        Self {
            enable_picking: true,
            enable_highlighting: true,
            enable_interacting: true,
            update_debug_cursor: true,
            print_debug_events: true,
        }
    }
}

fn simple_criteria(flag: bool) -> ShouldRun {
    if flag {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(Default)]
pub struct PausedForBlockers(pub(crate) bool);
impl PausedForBlockers {
    pub fn is_paused(&self) -> bool {
        self.0
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

impl PluginGroup for DefaultPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PickingPlugin);
        group.add(InteractablePickingPlugin);
        group.add(HighlightablePickingPlugin);
    }
}

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingPluginsState>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_picking)
                    })
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
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_interacting)
                    })
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
        let plugin = CustomHighlightablePickingPlugin(PhantomData::<(
            StandardMaterial,
            StandardMaterialPickingColors,
        )>::default());
        plugin.build(app)
    }
}

#[derive(Default)]
pub struct CustomHighlightablePickingPlugin<T, U>(PhantomData<(T, U)>);

impl<T, U> Plugin for CustomHighlightablePickingPlugin<T, U>
where
    T: Asset + Default,
    U: 'static + FromWorldHelper<T> + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshButtonMaterials<T, U>>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_highlighting)
                    })
                    .with_system(
                        get_initial_mesh_button_material::<T>
                            .after(PickingSystem::UpdateRaycast)
                            .before(PickingSystem::Highlighting),
                    )
                    .with_system(
                        mesh_highlighting::<T, U>
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
                .with_run_criteria(|state: Res<PickingPluginsState>| {
                    simple_criteria(state.update_debug_cursor)
                })
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
                .with_run_criteria(|state: Res<PickingPluginsState>| {
                    simple_criteria(state.print_debug_events)
                })
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
    pub pickable_button: PickableButton<StandardMaterial>,
    pub selection: Selection,
    pub hover: Hover,
}
