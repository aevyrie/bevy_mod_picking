mod events;
mod focus;
mod highlight;
mod selection;

use std::marker::PhantomData;

pub use crate::{
    events::{event_debug_system, picking_events_system, HoverEvent, PickingEvent, SelectionEvent},
    focus::{pause_for_picking_blockers, update_focus, Hover, PickingBlocker},
    highlight::{highlight_assets, DefaultHighlighting, Highlightable, Highlighting},
    selection::{update_selection, NoDeselect, Selection},
};

use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy};
use highlight::{get_initial_highlight_asset, Highlight};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickingSystem {
    UpdatePickSourcePositions,
    InitialHighlights,
    Highlighting,
    Selection,
    PauseForBlockers,
    Focus,
    Events,
}

/// Resource used to track the entity currently under the cursor. This is the primary input the the
/// picking plugin.
#[derive(Debug, Default, Clone)]
pub struct PickingTarget {
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone)]
pub struct PickingSettings {
    pub enable_picking: bool,
    pub enable_highlighting: bool,
    pub enable_interacting: bool,
}

impl Default for PickingSettings {
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
        group.add(InteractablePickingPlugin);
        HighlightablePickingPlugins.build(group);
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
                    .with_run_criteria(|state: Res<PickingSettings>| {
                        simple_criteria(state.enable_interacting)
                    })
                    .with_system(pause_for_picking_blockers.label(PickingSystem::PauseForBlockers))
                    .with_system(
                        update_focus
                            .label(PickingSystem::Focus)
                            .after(PickingSystem::PauseForBlockers),
                    )
                    .with_system(
                        update_selection
                            .label(PickingSystem::Selection)
                            .after(PickingSystem::Focus),
                    )
                    .with_system(
                        picking_events_system
                            .label(PickingSystem::Events)
                            .after(PickingSystem::Selection),
                    ),
            );
    }
}

pub struct HighlightablePickingPlugins;
impl PluginGroup for HighlightablePickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CustomHighlightPlugin::<StandardMaterial>::default());
        group.add(CustomHighlightPlugin::<ColorMaterial>::default());
    }
}

/// A highlighting plugin, generic over any asset that might be used for rendering the different
/// highlighting states.
#[derive(Default)]
pub struct CustomHighlightPlugin<T: 'static + Highlightable + Sync + Send>(PhantomData<T>);

impl<T> Plugin for CustomHighlightPlugin<T>
where
    T: 'static + Highlightable + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultHighlighting<T>>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingSettings>| {
                        simple_criteria(state.enable_highlighting)
                    })
                    .with_system(
                        get_initial_highlight_asset::<T>
                            .label(PickingSystem::InitialHighlights)
                            .before(PickingSystem::Highlighting),
                    )
                    .with_system(
                        highlight_assets::<T>
                            .label(PickingSystem::Highlighting)
                            .before(PickingSystem::Events),
                    ),
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
pub struct PickingSourceBundle {
    pub source: PickingSource,
    pub update: UpdatePicks,
}

impl Default for PickingSourceBundle {
    fn default() -> Self {
        PickingSourceBundle {
            source: PickingSource::new(),
            update: UpdatePicks::default(),
        }
    }
}

#[derive(Bundle, Default)]
pub struct PickableBundle {
    pub pickable_mesh: PickableTarget,
    pub interaction: Interaction,
    pub focus_policy: FocusPolicy,
    pub highlight: Highlight,
    pub selection: Selection,
    pub hover: Hover,
}
