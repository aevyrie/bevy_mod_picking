mod events;
mod focus;
mod highlight;
mod selection;

use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy};
use highlight::{get_initial_highlight_asset, Highlight};
use std::marker::PhantomData;

pub use crate::{
    events::{event_debug_system, update_events, HoverEvent, PickingEvent, SelectionEvent},
    focus::{update_focus, Hover},
    highlight::{highlight_assets, DefaultHighlighting, Highlightable, Highlighting},
    selection::{update_selection, NoDeselect, Selection},
};

/// Marks an entity that can be picked with this plugin.
#[derive(Debug, Clone, Default, Component)]
pub struct PickableTarget;

/// Typestates that represent the modular picking pipeline.
///
/// input systems -> produce `Cursor`s -> picking backend -> produce `CursorHit`s -> focus system
use self::{
    hit::CursorHit,
    input::{CursorId, CursorInput},
    interaction::CursorSelection,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickStage {
    /// Produces [`Cursor`]s.
    Input,
    /// Reads [`Cursor`]s and Produces [`CursorHit`]s.
    Backend,
    /// Reads [`CursorHit`]s, and determines focus, selection, and highlighting states.
    Focus,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum CorePickingSystem {
    UpdatePickSourcePositions,
    InitialHighlights,
    Highlighting,
    Selection,
    Focus,
    Events,
}

#[derive(Bundle)]
pub struct CursorBundle {
    pub id: CursorId,
    pub cursor: CursorInput,
    pub hit: CursorHit,
    pub selection: CursorSelection,
}
impl CursorBundle {
    pub fn new(id: CursorId, cursor: CursorInput) -> Self {
        CursorBundle {
            id,
            cursor,
            hit: CursorHit::default(),
            selection: CursorSelection::default(),
        }
    }
}

/// Information passed from  `bevy_picking_input` to the backend(s). This identifies all cursor inputs.
pub mod input {
    use bevy::{prelude::*, reflect::Uuid, render::camera::RenderTarget};

    /// Represents an input cursor used for picking.
    #[derive(Debug, Clone, Component, PartialEq)]
    pub struct CursorInput {
        pub enabled: bool,
        pub target: RenderTarget,
        pub position: Vec2,
        pub clicked: bool,
        pub multiselect: bool,
    }
    impl CursorInput {
        #[inline]
        pub fn is_in_viewport(&self, camera: &Camera) -> bool {
            camera
                .logical_viewport_rect()
                .map(|(min, max)| {
                    (self.position - min).min_element() >= 0.0
                        && (self.position - max).max_element() <= 0.0
                })
                .unwrap_or(false)
        }

        #[inline]
        pub fn is_same_target(&self, camera: &Camera) -> bool {
            camera.target == self.target
        }
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Component)]
    pub enum CursorId {
        Touch(u64),
        Mouse,
        Other(Uuid),
    }
    impl CursorId {
        pub fn is_touch(&self) -> bool {
            matches!(self, CursorId::Touch(_))
        }
        pub fn is_mouse(&self) -> bool {
            matches!(self, CursorId::Mouse)
        }
        pub fn is_other(&self) -> bool {
            matches!(self, CursorId::Other(_))
        }
    }
}

/// Information passed from the backend(s) to the focus system in [`bevy_picking_core`]. This
/// tells us what Entities have been hovered over by each cursor.
pub mod hit {
    use bevy::prelude::*;

    /// The entities currently under this entity's [`Cursor`](super::cursor::Cursor), if any,
    /// sorted from closest to farthest.
    ///
    /// For most cases, there will either be zero or one. For
    /// contexts like UI, it is often useful for picks to pass through to items below another
    /// item, so multiple entities may be picked at a given time.
    #[derive(Debug, Clone, Component, Default)]
    pub struct CursorHit {
        pub entities: Vec<Entity>,
    }
}

pub mod interaction {
    use bevy::prelude::*;

    #[derive(Debug, Clone, Component, Default)]
    pub struct CursorSelection {
        pub entities: Vec<Entity>,
    }
}

#[derive(Debug, Clone)]
pub struct PickingSettings {
    pub enable_backend: bool,
    pub enable_highlighting: bool,
    pub enable_interacting: bool,
}

impl Default for PickingSettings {
    fn default() -> Self {
        Self {
            enable_backend: true,
            enable_highlighting: true,
            enable_interacting: true,
        }
    }
}

pub fn simple_criteria(flag: bool) -> ShouldRun {
    if flag {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub struct CorePickingPlugin;
impl Plugin for CorePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingSettings>();
    }
}

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickingEvent>().add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .after(PickStage::Backend)
                .label(PickStage::Focus)
                .with_run_criteria(|state: Res<PickingSettings>| {
                    simple_criteria(state.enable_interacting)
                })
                .with_system(update_focus.label(CorePickingSystem::Focus))
                .with_system(
                    update_selection
                        .label(CorePickingSystem::Selection)
                        .after(CorePickingSystem::Focus),
                )
                .with_system(
                    update_events
                        .label(CorePickingSystem::Events)
                        .after(CorePickingSystem::Selection),
                ),
        );
    }
}

pub struct HighlightingPlugins;
impl PluginGroup for HighlightingPlugins {
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
                    .after(PickStage::Backend)
                    .label(PickStage::Focus)
                    .with_run_criteria(|state: Res<PickingSettings>| {
                        simple_criteria(state.enable_highlighting)
                    })
                    .with_system(
                        get_initial_highlight_asset::<T>
                            .label(CorePickingSystem::InitialHighlights)
                            .before(CorePickingSystem::Highlighting),
                    )
                    .with_system(
                        highlight_assets::<T>
                            .label(CorePickingSystem::Highlighting)
                            .before(CorePickingSystem::Events),
                    ),
            );
    }
}

pub struct DebugEventsPlugin;
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::Last,
            event_debug_system.after(CorePickingSystem::Events),
        );
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

pub trait IntoShouldRun {
    fn should_run(&self) -> ShouldRun;
}
impl IntoShouldRun for bool {
    fn should_run(&self) -> ShouldRun {
        if *self {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }
}
