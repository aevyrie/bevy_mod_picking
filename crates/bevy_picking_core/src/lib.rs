#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod backend;
mod focus;
mod highlight;
pub mod input;
pub mod output;
mod selection;

use bevy::{ecs::schedule::ShouldRun, prelude::*, reflect::Uuid, ui::FocusPolicy};
use focus::{send_click_events, PickLayer};
use highlight::PickHighlight;
use input::{PointerLocation, PointerMultiselect, PointerPress};
use output::{PickInteraction, PointerInteraction};
use selection::PointerSelectionEvent;

pub use crate::{
    focus::update_focus,
    highlight::{
        update_highlight_assets, CustomHighlightingPlugin, DefaultHighlighting, Highlightable,
        HighlightingPlugins, InitialHighlight,
    },
    selection::{send_selection_events, NoDeselect, PickSelection},
};

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    pub pick_layer: PickLayer,
    pub interaction: PickInteraction,
    pub selection: PickSelection,
    pub highlight: PickHighlight,
    pub focus_policy: FocusPolicy,
}

#[derive(Bundle)]
pub struct PointerBundle {
    pub id: PointerId,
    pub location: input::PointerLocation,
    pub click: input::PointerPress,
    pub multi_select: input::PointerMultiselect,
    pub interaction: output::PointerInteraction,
}
impl PointerBundle {
    pub fn new(id: PointerId) -> Self {
        PointerBundle {
            id,
            location: PointerLocation::default(),
            click: PointerPress::default(),
            multi_select: PointerMultiselect::default(),
            interaction: PointerInteraction::default(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickStage {
    /// Produces [`input::PointerPressEvent`]s, [`input::PointerLocationEvent`]s, and updates
    /// [`PointerMultiselect`].
    Input,
    /// Reads inputs and produces [`backend::PointerOverEvent`]s.
    Backend,
    /// Reads [`backend::PointerOverEvent`]s, and updates focus, selection, and highlighting states.
    Events,
    ///
    EventListeners,
}

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingSettings>()
            .add_event::<input::PointerPressEvent>()
            .add_event::<input::PointerLocationEvent>()
            .add_event::<backend::PointerOverEvent>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Input)
                    .before(PickStage::Backend)
                    .with_system(input::PointerLocationEvent::receive)
                    .with_system(input::PointerPressEvent::receive),
            );
    }
}

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<output::PointerOver>()
            .add_event::<output::PointerOut>()
            .add_event::<output::PointerEnter>()
            .add_event::<output::PointerLeave>()
            .add_event::<output::PointerDown>()
            .add_event::<output::PointerUp>()
            .add_event::<output::PointerClick>()
            .add_event::<output::PointerMove>()
            .add_event::<output::PointerCancel>()
            .add_event::<PointerSelectionEvent>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Backend)
                    .label(PickStage::Events)
                    .with_run_criteria(|state: Res<PickingSettings>| state.interacting)
                    .with_system(update_focus)
                    .with_system(PickInteraction::update.after(update_focus))
                    .with_system(send_click_events.after(update_focus))
                    .with_system(send_selection_events.after(send_click_events))
                    .with_system(PointerSelectionEvent::receive.after(send_selection_events)),
            )
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Events)
                    .label(PickStage::EventListeners)
                    .with_system(output::PointerOver::event_bubbling)
                    .with_system(output::PointerOut::event_bubbling)
                    .with_system(output::PointerEnter::event_bubbling)
                    .with_system(output::PointerLeave::event_bubbling)
                    .with_system(output::PointerDown::event_bubbling)
                    .with_system(output::PointerUp::event_bubbling)
                    .with_system(output::PointerClick::event_bubbling)
                    .with_system(output::PointerMove::event_bubbling)
                    .with_system(output::PointerCancel::event_bubbling),
            );
    }
}

pub struct DebugEventsPlugin;
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(event_debug_system::<output::PointerOver>)
                .with_system(event_debug_system::<output::PointerOut>)
                .with_system(event_debug_system::<output::PointerEnter>)
                .with_system(event_debug_system::<output::PointerLeave>)
                .with_system(event_debug_system::<output::PointerDown>)
                .with_system(event_debug_system::<output::PointerUp>)
                .with_system(event_debug_system::<output::PointerClick>)
                .with_system(event_debug_system::<output::PointerMove>),
        );
    }
}

pub struct DefaultPointersPlugin;
impl Plugin for DefaultPointersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(PointerId::add_default_pointers);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Component, Reflect)]
pub enum PointerId {
    Touch(u64),
    Mouse,
    Other(Uuid),
}
impl PointerId {
    pub fn is_touch(&self) -> bool {
        matches!(self, PointerId::Touch(_))
    }
    pub fn is_mouse(&self) -> bool {
        matches!(self, PointerId::Mouse)
    }
    pub fn is_other(&self) -> bool {
        matches!(self, PointerId::Other(_))
    }
    pub fn add_default_pointers(mut commands: Commands) {
        commands.spawn_bundle(PointerBundle::new(PointerId::Mouse));
        // Windows was the highest amount I could find at 20 touch + 10 writing
        for i in 0..30 {
            commands.spawn_bundle(PointerBundle::new(PointerId::Touch(i)));
        }
    }
}

#[derive(Debug, Clone)]
pub struct PickingSettings {
    pub backend: ShouldRun,
    pub highlighting: ShouldRun,
    pub interacting: ShouldRun,
}

impl Default for PickingSettings {
    fn default() -> Self {
        Self {
            backend: ShouldRun::Yes,
            highlighting: ShouldRun::Yes,
            interacting: ShouldRun::Yes,
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system<E: output::IsPointerEvent>(mut events: EventReader<E>) {
    for event in events.iter() {
        info!(
            "{event}, Event: {}",
            std::any::type_name::<E>().split("::").last().unwrap()
        );
    }
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
