mod events;
mod focus;
mod highlight;
mod selection;

use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy};
use highlight::{get_initial_highlight_asset, Highlight};
use picking::CoreSystem;
use std::marker::PhantomData;

pub use crate::{
    events::{event_debug_system, update_events, HoverEvent, PickingEvent, SelectionEvent},
    focus::{pause_for_picking_blockers, update_focus, Hover, PickingBlocker},
    highlight::{highlight_assets, DefaultHighlighting, Highlightable, Highlighting},
    selection::{update_selection, NoDeselect, Selection},
};

/// Marks an entity that can be picked with this plugin.
#[derive(Debug, Clone, Default, Component)]
pub struct PickableTarget;

/// Typestates that represent the modular picking pipeline.
///
/// input systems -> produce `Cursor`s -> picking backend -> produce `Hit`s -> focus system
pub mod picking {
    use bevy::prelude::{StageLabel, SystemLabel, *};

    use self::{
        cursor::{Cursor, CursorId},
        hit::CursorHit,
    };

    #[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
    pub enum Stage {
        /// Produces [`Cursor`] events.
        Input,
        /// Consumes [`Cursor`] events and Produces [`Hit`] events.
        Backend,
        /// Consumes [`Hit`] events, and determines focus, selection, and highlighting states.
        Picking,
    }

    #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
    pub enum CoreSystem {
        UpdatePickSourcePositions,
        InitialHighlights,
        Highlighting,
        Selection,
        PauseForBlockers,
        Focus,
        Events,
    }

    #[derive(Bundle)]
    pub struct CursorBundle {
        pub id: CursorId,
        pub cursor: Cursor,
        pub hit: CursorHit,
    }
    impl CursorBundle {
        pub fn new(id: CursorId, cursor: Cursor) -> Self {
            CursorBundle {
                id,
                cursor,
                hit: CursorHit::default(),
            }
        }
    }

    /// Information passed from  [`bevy_picking_input`] to the backend(s). This identifies all cursor inputs.
    pub mod cursor {
        use bevy::{prelude::*, reflect::Uuid, render::camera::RenderTarget};

        /// Represents an input cursor used for picking.
        #[derive(Debug, Clone, Component, PartialEq)]
        pub struct Cursor {
            pub enabled: bool,
            pub clicked: bool,
            pub target: RenderTarget,
            pub position: Vec2,
        }

        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Component)]
        pub enum CursorId {
            Touch(u64),
            Mouse,
            Other(Uuid),
        }

        #[derive(Debug, Clone, Eq, PartialEq, Default)]
        pub struct MultiSelect {
            pub active: bool,
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
            pub hit_entities: Vec<Entity>,
        }
    }
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

pub fn simple_criteria(flag: bool) -> ShouldRun {
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

pub struct CorePickingPlugin;
impl Plugin for CorePickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingSettings>()
            .init_resource::<picking::cursor::MultiSelect>();
    }
}

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PausedForBlockers>()
            .add_event::<PickingEvent>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .with_run_criteria(|state: Res<PickingSettings>| {
                        simple_criteria(state.enable_interacting)
                    })
                    .with_system(pause_for_picking_blockers.label(CoreSystem::PauseForBlockers))
                    .with_system(
                        update_focus
                            .label(CoreSystem::Focus)
                            .after(CoreSystem::PauseForBlockers),
                    )
                    .with_system(
                        update_selection
                            .label(CoreSystem::Selection)
                            .after(CoreSystem::Focus),
                    )
                    .with_system(
                        update_events
                            .label(CoreSystem::Events)
                            .after(CoreSystem::Selection),
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
                    .with_run_criteria(|state: Res<PickingSettings>| {
                        simple_criteria(state.enable_highlighting)
                    })
                    .with_system(
                        get_initial_highlight_asset::<T>
                            .label(CoreSystem::InitialHighlights)
                            .before(CoreSystem::Highlighting),
                    )
                    .with_system(
                        highlight_assets::<T>
                            .label(CoreSystem::Highlighting)
                            .before(CoreSystem::Events),
                    ),
            );
    }
}

pub struct DebugEventsPlugin;
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            event_debug_system.after(CoreSystem::Events),
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
