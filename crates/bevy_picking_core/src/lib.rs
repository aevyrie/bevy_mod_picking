//! Core functionality and types required for `bevy_mod_picking` to function.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

pub mod backend;
pub mod focus;
pub mod output;
pub mod pointer;

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use focus::update_focus;
use output::{
    event_bubbling, interactions_from_events, pointer_events, send_click_and_drag_events,
    send_drag_over_events,
};

/// Groups the stages of the picking process under shared labels.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PickStage {
    /// Produces pointer input events.
    Input,
    /// Reads inputs and produces [`backend::EntitiesUnderPointer`]s.
    Backend,
    /// Reads [`backend::EntitiesUnderPointer`]s, and updates focus, selection, and highlighting states.
    Focus,
    /// Updates event listeners and bubbles [`output::PointerEvent`]s
    EventListeners,
}

/// Receives input events, and provides the shared types used by other picking plugins.
pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<pointer::InputPress>()
            .add_event::<pointer::InputMove>()
            .add_event::<backend::EntitiesUnderPointer>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Input)
                    .before(PickStage::Backend)
                    .with_system(pointer::InputMove::receive)
                    .with_system(pointer::InputPress::receive),
            );
    }
}

/// Generates [`PointerEvent`](output::PointerEvent)s and handles event bubbling.
pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<focus::HoverMap>()
            .init_resource::<output::DragMap>()
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
            .add_event::<output::PointerDrag>()
            .add_event::<output::PointerDragEnd>()
            .add_event::<output::PointerDragEnter>()
            .add_event::<output::PointerDragOver>()
            .add_event::<output::PointerDragLeave>()
            .add_event::<output::PointerDrop>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Backend)
                    .label(PickStage::Focus)
                    // Focus
                    .with_system(update_focus)
                    .with_system(pointer_events.after(update_focus))
                    // Output
                    .with_system(interactions_from_events.after(pointer_events))
                    .with_system(send_click_and_drag_events.after(pointer_events))
                    .with_system(send_drag_over_events.after(send_click_and_drag_events)),
            )
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Focus)
                    .label(PickStage::EventListeners)
                    .with_system(event_bubbling::<output::Over>)
                    .with_system(event_bubbling::<output::Out>)
                    // PointerEnter does not allow bubbling
                    // PointerLeave does not allow bubbling
                    .with_system(event_bubbling::<output::Down>)
                    .with_system(event_bubbling::<output::Up>)
                    .with_system(event_bubbling::<output::Click>)
                    .with_system(event_bubbling::<output::Move>)
                    .with_system(event_bubbling::<output::Cancel>)
                    .with_system(event_bubbling::<output::DragStart>)
                    .with_system(event_bubbling::<output::Drag>)
                    .with_system(event_bubbling::<output::DragEnd>)
                    .with_system(event_bubbling::<output::DragEnter>)
                    .with_system(event_bubbling::<output::DragOver>)
                    .with_system(event_bubbling::<output::DragLeave>)
                    .with_system(event_bubbling::<output::Drop>),
            );
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

/// Simple trait used to convert a boolean to a run criteria.
trait IntoShouldRun {
    /// Converts `self` into [`ShouldRun`].
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
