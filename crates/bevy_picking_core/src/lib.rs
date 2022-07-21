#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod backend;
pub mod focus;
pub mod input;
pub mod output;

use bevy::{ecs::schedule::ShouldRun, prelude::*, reflect::Uuid};
use focus::{pointer_events, update_focus};
use output::{event_bubbling, send_click_and_drag_events, PickInteraction};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickStage {
    /// Produces [`input::PointerPressEvent`]s, [`input::PointerLocationEvent`]s, and updates
    /// [`PointerMultiselect`].
    Input,
    /// Reads inputs and produces [`backend::EntitiesUnderPointer`]s.
    Backend,
    /// Reads [`backend::EntitiesUnderPointer`]s, and updates focus, selection, and highlighting states.
    Focus,
    /// Updates event listeners and bubbles [`output::PointerEvent`]s
    EventListeners,
}

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingSettings>()
            .add_event::<input::InputPress>()
            .add_event::<input::InputMove>()
            .add_event::<backend::EntitiesUnderPointer>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Input)
                    .before(PickStage::Backend)
                    .with_system(input::InputMove::receive)
                    .with_system(input::InputPress::receive),
            );
    }
}

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<focus::HoverMap>()
            .add_event::<output::PointerOver>()
            .add_event::<output::PointerOut>()
            .add_event::<output::PointerEnter>()
            .add_event::<output::PointerLeave>()
            .add_event::<output::PointerDown>()
            .add_event::<output::PointerUp>()
            .add_event::<output::PointerClick>()
            .add_event::<output::PointerMove>()
            .add_event::<output::PointerCancel>()
            .add_event::<output::PointerDragStart>()
            .add_event::<output::PointerDragEnd>()
            .add_event::<output::PointerDrag>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Backend)
                    .label(PickStage::Focus)
                    .with_run_criteria(|state: Res<PickingSettings>| state.interacting)
                    // Focus
                    .with_system(update_focus)
                    .with_system(pointer_events.after(update_focus))
                    // Output
                    .with_system(PickInteraction::update_from_events.after(pointer_events))
                    .with_system(send_click_and_drag_events.after(update_focus)),
            )
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Focus)
                    .label(PickStage::EventListeners)
                    .with_system(event_bubbling::<output::Over>)
                    .with_system(event_bubbling::<output::Out>)
                    .with_system(event_bubbling::<output::Enter>)
                    .with_system(event_bubbling::<output::Leave>)
                    .with_system(event_bubbling::<output::Down>)
                    .with_system(event_bubbling::<output::Up>)
                    .with_system(event_bubbling::<output::Click>)
                    .with_system(event_bubbling::<output::Move>)
                    .with_system(event_bubbling::<output::Cancel>)
                    .with_system(event_bubbling::<output::DragStart>)
                    .with_system(event_bubbling::<output::DragEnd>)
                    .with_system(event_bubbling::<output::Drag>),
            );
    }
}

pub struct DebugEventsPlugin;
impl Plugin for DebugEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(event_debug::<output::PointerOver>)
                .with_system(event_debug::<output::PointerOut>)
                .with_system(event_debug::<output::PointerEnter>)
                .with_system(event_debug::<output::PointerLeave>)
                .with_system(event_debug::<output::PointerDown>)
                .with_system(event_debug::<output::PointerUp>)
                .with_system(event_debug::<output::PointerClick>)
                .with_system(event_debug::<output::PointerMove>)
                .with_system(event_debug::<output::PointerCancel>)
                .with_system(event_debug::<output::PointerDragStart>)
                .with_system(event_debug::<output::PointerDragEnd>)
                .with_system(event_debug::<output::PointerDrag>),
        );
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

/// Listens for pointer events of type `E` and prints them
pub fn event_debug<E: output::IsPointerEvent>(mut events: EventReader<E>) {
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
