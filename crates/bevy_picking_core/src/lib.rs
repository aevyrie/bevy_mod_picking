mod events;
mod focus;
mod highlight;
mod selection;

use bevy::{app::PluginGroupBuilder, ecs::schedule::ShouldRun, prelude::*, ui::FocusPolicy};
use highlight::Highlight;
use input::CursorClick;
use interaction::CursorInteraction;

pub use crate::{
    events::{event_debug_system, CursorEvent},
    focus::update_focus,
    highlight::{highlight_assets, DefaultHighlighting, Highlightable, Highlighting},
    selection::{update_selection, NoDeselect, Selection},
};

/// Marks an entity that can be picked with this plugin.
#[derive(Debug, Clone, Default, Component)]
pub struct PickableTarget;

/// Typestates that represent the modular picking pipeline.
///
/// input systems -> produce `Cursor`s -> picking backend -> produce `CursorOver`s -> focus system
use self::{
    backend::CursorOver,
    input::{CursorId, CursorLocation},
    interaction::CursorSelection,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickStage {
    /// Produces [`CursorInput`]s.
    Input,
    /// Reads [`CursorInput`]s and Produces [`CursorOver`]s.
    Backend,
    /// Reads [`CursorOver`]s, and determines focus, selection, and highlighting states.
    Focus,
}

#[derive(Bundle)]
pub struct CursorBundle {
    pub id: CursorId,
    pub click: CursorClick,
    pub cursor: CursorLocation,
    pub hit: CursorOver,
    pub selection: CursorSelection,
}
impl CursorBundle {
    pub fn new(id: CursorId, cursor: CursorLocation, click: CursorClick) -> Self {
        CursorBundle {
            id,
            cursor,
            click,
            hit: CursorOver::default(),
            selection: CursorSelection::default(),
        }
    }
}

/// Information passed from  `bevy_picking_input` to the backend(s). This identifies all cursor inputs.
pub mod input {
    use bevy::{prelude::*, reflect::Uuid, render::camera::RenderTarget};

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Component, Reflect)]
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

    #[derive(Debug, Clone, Component, PartialEq)]
    pub struct Location {
        pub target: RenderTarget,
        pub position: Vec2,
    }

    /// Represents an input cursor used for picking.
    #[derive(Debug, Clone, Component, PartialEq)]
    pub struct CursorLocation {
        pub location: Option<Location>,
    }
    impl CursorLocation {
        #[inline]
        pub fn is_in_viewport(&self, camera: &Camera) -> bool {
            if let Some(loc) = &self.location {
                camera
                    .logical_viewport_rect()
                    .map(|(min, max)| {
                        (loc.position - min).min_element() >= 0.0
                            && (loc.position - max).max_element() <= 0.0
                    })
                    .unwrap_or(false)
            } else {
                false
            }
        }

        #[inline]
        pub fn is_same_target(&self, camera: &Camera) -> bool {
            self.location
                .as_ref()
                .map(|loc| loc.target == camera.target)
                .unwrap_or(false)
        }
    }

    #[derive(Debug, Default, Clone, Component, PartialEq)]
    pub struct CursorClick {
        pub is_clicked: bool,
    }

    #[derive(Debug, Default, Clone, Component, PartialEq)]
    pub struct CursorMultiSelect {
        pub is_clicked: bool,
    }
}

/// Information passed from the backend(s) to the focus system in [`bevy_picking_core`]. This
/// tells us what Entities have been hovered over by each cursor.
pub mod backend {
    use bevy::prelude::*;

    /// The entities currently under this entity's [`Cursor`](super::cursor::Cursor), if any,
    /// sorted from closest to farthest.
    ///
    /// For most cases, there will either be zero or one. For
    /// contexts like UI, it is often useful for picks to pass through to items below another
    /// item, so multiple entities may be picked at a given time.
    #[derive(Debug, Clone, Component, Default)]
    pub struct CursorOver {
        pub entities: Vec<Entity>,
    }
    impl CursorOver {
        pub fn clear(&mut self) {
            self.entities.clear();
        }

        pub fn entities(&self) -> &[Entity] {
            self.entities.as_ref()
        }
    }
}

pub mod interaction {
    use bevy::{prelude::*, utils::hashbrown::HashSet};

    use crate::input::CursorId;

    #[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
    pub struct CursorInteraction {
        pub(crate) hovered: HashSet<CursorId>,
        pub(crate) clicked: HashSet<CursorId>,
    }
    impl CursorInteraction {
        pub fn is_hovered(&self, cursor: &CursorId) -> bool {
            self.hovered.contains(cursor)
        }

        pub fn is_clicked(&self, cursor: &CursorId) -> bool {
            self.clicked.contains(cursor)
        }

        pub fn is_hovered_any(&self) -> bool {
            !self.hovered.is_empty()
        }

        pub fn is_clicked_any(&self) -> bool {
            !self.clicked.is_empty()
        }
    }

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
        app.add_event::<CursorEvent>().add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .after(PickStage::Backend)
                .label(PickStage::Focus)
                .with_run_criteria(|state: Res<PickingSettings>| {
                    simple_criteria(state.enable_interacting)
                })
                .with_system(update_focus)
                .with_system(update_selection.after(update_focus)),
        );
    }
}

pub struct HighlightingPlugins;
impl PluginGroup for HighlightingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(highlight::CustomHighlightPlugin::<StandardMaterial>::default());
        group.add(highlight::CustomHighlightPlugin::<ColorMaterial>::default());
    }
}

pub struct DebugEventsPlugin;
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, event_debug_system.after(PickStage::Focus));
    }
}

#[derive(Bundle, Default)]
pub struct PickableBundle {
    pub pickable_mesh: PickableTarget,
    pub focus_policy: FocusPolicy,
    pub interaction: CursorInteraction,
    pub selection: Selection,
    pub highlight: Highlight,
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
