pub mod events;
pub mod focus;
pub mod highlight;
pub mod mouse;
pub mod selection;

pub use crate::{
    events::{event_debug_system, mesh_events_system, HoverEvent, PickingEvent, SelectionEvent},
    focus::{mesh_focus, pause_for_picking_blockers, Hover, PickingBlocker},
    highlight::{mesh_highlighting, DefaultHighlighting, Highlighting},
    mouse::update_pick_source_positions,
    selection::{mesh_selection, NoDeselect, Selection},
};
pub use bevy_mod_raycast::{Primitive3d, RaycastMesh, RaycastSource};

use bevy::{app::PluginGroupBuilder, asset::Asset, prelude::*, ui::FocusPolicy};
use highlight::{get_initial_mesh_highlight_asset, Highlight};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PickingSystem {
    UpdateIntersections,
    Events,
    MeshHighlighting,
}

/// A type alias for the concrete [RaycastMesh](bevy_mod_raycast::RaycastMesh) type used for Picking.
pub type PickableMesh = RaycastMesh<PickingRaycastSet>;
/// A type alias for the concrete [RaycastSource](bevy_mod_raycast::RaycastSource) type used for Picking.
pub type PickingCamera = RaycastSource<PickingRaycastSet>;

/// This unit struct is used to tag the generic ray casting types `RaycastMesh` and
/// `RaycastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RaycastMesh<PickingRaycastSet>`
/// vs. `RaycastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
pub struct PickingRaycastSet;

#[derive(Clone, Debug, Resource)]
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

#[derive(Default, Resource)]
pub struct PausedForBlockers(pub(crate) bool);
impl PausedForBlockers {
    pub fn is_paused(&self) -> bool {
        self.0
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
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
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PickingPlugin)
            .add(InteractablePickingPlugin)
            .add(CustomHighlightPlugin::<StandardMaterial> {
                highlighting_default: |mut assets| DefaultHighlighting {
                    hovered: assets.add(Color::rgb(0.35, 0.35, 0.35).into()),
                    pressed: assets.add(Color::rgb(0.35, 0.75, 0.35).into()),
                    selected: assets.add(Color::rgb(0.35, 0.35, 0.75).into()),
                },
            })
            .add(CustomHighlightPlugin::<ColorMaterial> {
                highlighting_default: |mut assets| DefaultHighlighting {
                    hovered: assets.add(Color::rgb(0.35, 0.35, 0.35).into()),
                    pressed: assets.add(Color::rgb(0.35, 0.75, 0.35).into()),
                    selected: assets.add(Color::rgb(0.35, 0.35, 0.75).into()),
                },
            })
    }
}

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingPluginsState>()
            .add_systems(
                (
                    update_pick_source_positions,
                    bevy_mod_raycast::build_rays::<PickingRaycastSet>,
                    bevy_mod_raycast::update_raycast::<PickingRaycastSet>,
                    bevy_mod_raycast::update_intersections::<PickingRaycastSet>,
                )
                    .chain()
                    .in_set(PickingSystem::UpdateIntersections),
            )
            .configure_set(
                PickingSystem::UpdateIntersections
                    .run_if(|state: Res<PickingPluginsState>| state.enable_picking)
                    .in_base_set(CoreSet::PreUpdate),
            );
    }
}

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PausedForBlockers>()
            .add_event::<PickingEvent>()
            .add_systems(
                (
                    pause_for_picking_blockers,
                    mesh_focus,
                    mesh_selection,
                    mesh_events_system,
                )
                    .chain()
                    .in_set(PickingSystem::Events),
            )
            .configure_set(
                PickingSystem::Events
                    .run_if(|state: Res<PickingPluginsState>| state.enable_interacting)
                    .in_base_set(CoreSet::PreUpdate)
                    .after(PickingSystem::UpdateIntersections),
            );
    }
}

/// A highlighting plugin, generic over any asset that might be used for rendering the different
/// highlighting states.
pub struct CustomHighlightPlugin<T: 'static + Asset + Sync + Send> {
    pub highlighting_default: fn(ResMut<Assets<T>>) -> DefaultHighlighting<T>,
}

impl<T> Plugin for CustomHighlightPlugin<T>
where
    T: 'static + Asset + Sync + Send,
{
    fn build(&self, app: &mut App) {
        let highlighting_default = self.highlighting_default;
        app.add_startup_system(move |mut commands: Commands, assets: ResMut<Assets<T>>| {
            commands.insert_resource(highlighting_default(assets));
        })
        .add_systems(
            (
                get_initial_mesh_highlight_asset::<T>,
                mesh_highlighting::<T>,
            )
                .chain()
                .in_set(PickingSystem::MeshHighlighting),
        )
        .configure_set(
            PickingSystem::MeshHighlighting
                .run_if(|state: Res<PickingPluginsState>| state.enable_highlighting)
                .in_base_set(CoreSet::PreUpdate)
                .before(PickingSystem::Events)
                .after(PickingSystem::UpdateIntersections),
        );
    }
}

pub struct DebugCursorPickingPlugin;
impl Plugin for DebugCursorPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            bevy_mod_raycast::update_debug_cursor::<PickingRaycastSet>
                .in_base_set(CoreSet::PreUpdate)
                .after(PickingSystem::UpdateIntersections),
        );
    }
}

pub struct DebugEventsPickingPlugin;
impl Plugin for DebugEventsPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            event_debug_system
                .in_base_set(CoreSet::PreUpdate)
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
    pub highlight: Highlight,
    pub selection: Selection,
    pub hover: Hover,
}
