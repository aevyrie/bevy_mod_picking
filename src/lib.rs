mod events;
mod focus;
mod highlight;
mod mouse;
mod selection;

pub use crate::{
    events::{event_debug_system, mesh_events_system, HoverEvent, PickingEvent, SelectionEvent},
    focus::{mesh_focus, pause_for_picking_blockers, Hover, PickingBlocker},
    highlight::{
        mesh_highlighting, DefaultHighlighting, Highlightable, Highlighting,
        StandardMaterialHighlight,
    },
    mouse::update_pick_source_positions,
    selection::{mesh_selection, NoDeselect, Selection},
};
pub use bevy_mod_raycast::{Primitive3d, RayCastSource};

use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy};
use highlight::{get_initial_mesh_highlight_asset, ColorMaterialHighlight, Highlight};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickingSystem {
    UpdatePickSourcePositions,
    BuildRays,
    UpdateRaycast,
    UpdateIntersections,
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
}

impl Default for PickingPluginsState {
    fn default() -> Self {
        Self {
            enable_picking: true,
            enable_highlighting: true,
            enable_interacting: true,
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
        HighlightablePickingPlugins.build(group);
    }
}

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingPluginsState>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_picking)
                    })
                    .with_system(
                        update_pick_source_positions
                            .label(PickingSystem::UpdatePickSourcePositions)
                            .before(PickingSystem::BuildRays),
                    )
                    .with_system(
                        bevy_mod_raycast::build_rays::<PickingRaycastSet>
                            .label(PickingSystem::BuildRays)
                            .before(PickingSystem::UpdateRaycast),
                    )
                    .with_system(
                        bevy_mod_raycast::update_raycast::<PickingRaycastSet>
                            .label(PickingSystem::UpdateRaycast)
                            .before(PickingSystem::UpdateIntersections),
                    )
                    .with_system(
                        bevy_mod_raycast::update_intersections::<PickingRaycastSet>
                            .label(PickingSystem::UpdateIntersections),
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
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_interacting)
                    })
                    .with_system(
                        pause_for_picking_blockers
                            .label(PickingSystem::PauseForBlockers)
                            .after(PickingSystem::UpdateIntersections),
                    )
                    .with_system(
                        mesh_focus
                            .label(PickingSystem::Focus)
                            .after(PickingSystem::PauseForBlockers),
                    )
                    .with_system(
                        mesh_selection
                            .label(PickingSystem::Selection)
                            .after(PickingSystem::Focus),
                    )
                    .with_system(
                        mesh_events_system
                            .label(PickingSystem::Events)
                            .after(PickingSystem::Selection),
                    ),
            );
    }
}

pub struct HighlightablePickingPlugins;
impl PluginGroup for HighlightablePickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CustomHighlightPlugin(StandardMaterialHighlight));
        group.add(CustomHighlightPlugin(ColorMaterialHighlight));
    }
}

/// A highlighting plugin, generic over any asset that might be used for rendering the different
/// highlighting states.
#[derive(Default)]
pub struct CustomHighlightPlugin<T: 'static + Highlightable + Sync + Send>(pub T);

impl<T> Plugin for CustomHighlightPlugin<T>
where
    T: 'static + Highlightable + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultHighlighting<T>>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingPluginsState>| {
                        simple_criteria(state.enable_highlighting)
                    })
                    .with_system(
                        get_initial_mesh_highlight_asset::<T::HighlightAsset>
                            .after(PickingSystem::UpdateIntersections)
                            .before(PickingSystem::Highlighting),
                    )
                    .with_system(
                        mesh_highlighting::<T>
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
            CoreStage::First,
            bevy_mod_raycast::update_debug_cursor::<PickingRaycastSet>
                .after(PickingSystem::UpdateIntersections),
        );
    }
}

pub struct DebugEventsPickingPlugin;
impl Plugin for DebugEventsPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            event_debug_system.after(PickingSystem::Events),
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
    pub highlight: Highlight,
    pub selection: Selection,
    pub hover: Hover,
}
