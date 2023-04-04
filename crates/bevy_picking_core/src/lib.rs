//! Core functionality and types required for `bevy_mod_picking` to function.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

pub mod backend;
pub mod debug;
pub mod focus;
pub mod output;
pub mod pointer;

use bevy::prelude::*;
use focus::update_focus;
use output::{
    event_bubbling, interactions_from_events, pointer_events, send_click_and_drag_events,
    send_drag_over_events,
};

/// Used to globally toggle picking features at runtime.
#[derive(Clone, Debug, Resource)]
pub struct PickingPluginsSettings {
    /// Enables and disables all picking features.
    pub enable: bool,
    /// Enables and disables input collection.
    pub enable_input: bool,
    /// Enables and disables entity highlighting.
    pub enable_highlighting: bool,
    /// Enables and disables updating interaction states of entities.
    pub enable_interacting: bool,
}

impl PickingPluginsSettings {
    /// Whether or not input collection systems should be running.
    pub fn input_enabled(state: Res<Self>) -> bool {
        state.enable_input && state.enable
    }
    /// Whether or not entity highlighting systems should be running.
    pub fn highlighting_should_run(state: Res<Self>) -> bool {
        state.enable_highlighting && state.enable
    }
    /// Whether or not systems updating entities' [`Interaction`] component should be running.
    pub fn interaction_should_run(state: Res<Self>) -> bool {
        state.enable_highlighting && state.enable
    }
}

impl Default for PickingPluginsSettings {
    fn default() -> Self {
        Self {
            enable: true,
            enable_input: true,
            enable_highlighting: true,
            enable_interacting: true,
        }
    }
}

/// Components needed to build a pointer. Multiple pointers can be active at once, with each pointer
/// being an entity.
///
/// `Mouse` and `Touch` pointers are automatically spawned as needed. Use this bundle if you are
/// spawning a custom `PointerId::Custom` pointer, either for testing, or as a software controller
/// pointer, or if you are replacing the default touch and mouse inputs.
#[derive(Bundle)]
pub struct PointerCoreBundle {
    /// The pointer's unique [`PointerId`](pointer::PointerId).
    pub id: pointer::PointerId,
    /// Tracks the pointer's location.
    pub location: pointer::PointerLocation,
    /// Tracks the pointer's button press state.
    pub click: pointer::PointerPress,
    /// Tracks the pointer's interaction state.
    pub interaction: output::PointerInteraction,
}

impl PointerCoreBundle {
    /// Sets the location of the pointer bundle
    pub fn with_location(mut self, location: pointer::Location) -> Self {
        self.location.location = Some(location);
        self
    }
}

impl PointerCoreBundle {
    /// Create a new pointer with the provided [`PointerId`](pointer::PointerId).
    pub fn new(id: pointer::PointerId) -> Self {
        PointerCoreBundle {
            id,
            location: pointer::PointerLocation::default(),
            click: pointer::PointerPress::default(),
            interaction: output::PointerInteraction::default(),
        }
    }
}

/// Groups the stages of the picking process under shared labels.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PickSet {
    /// Produces pointer input events.
    Input,
    /// Runs after input events are generated but before commands are flushed.
    PostInput,
    /// Receives and processes pointer input events.
    ProcessInput,
    /// Reads inputs and produces [`backend::EntitiesUnderPointer`]s.
    Backend,
    /// Reads [`backend::EntitiesUnderPointer`]s, and updates focus, selection, and highlighting
    /// states.
    Focus,
    /// Updates event listeners and bubbles [`output::PointerEvent`]s
    EventListeners,
}

/// Receives input events, and provides the shared types used by other picking plugins.
pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingPluginsSettings>()
            .init_resource::<pointer::PointerMap>()
            .add_event::<pointer::InputPress>()
            .add_event::<pointer::InputMove>()
            .add_event::<backend::EntitiesUnderPointer>()
            .add_system(pointer::update_pointer_map.in_set(PickSet::Input))
            .add_systems(
                (pointer::InputMove::receive, pointer::InputPress::receive)
                    .in_set(PickSet::ProcessInput),
            );
    }
}

/// Generates [`PointerEvent`](output::PointerEvent)s and handles event bubbling.
pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<focus::HoverMap>()
            .init_resource::<focus::PreviousHoverMap>()
            .init_resource::<output::DragMap>()
            .add_event::<output::PointerCancel>()
            .add_event::<output::PointerOver>()
            .add_event::<output::PointerOut>()
            .add_event::<output::PointerDown>()
            .add_event::<output::PointerUp>()
            .add_event::<output::PointerClick>()
            .add_event::<output::PointerMove>()
            .add_event::<output::PointerDragStart>()
            .add_event::<output::PointerDrag>()
            .add_event::<output::PointerDragEnd>()
            .add_event::<output::PointerDragEnter>()
            .add_event::<output::PointerDragOver>()
            .add_event::<output::PointerDragLeave>()
            .add_event::<output::PointerDrop>()
            .add_systems(
                (
                    update_focus,
                    pointer_events,
                    interactions_from_events,
                    send_click_and_drag_events,
                    send_drag_over_events,
                )
                    .chain()
                    .in_set(PickSet::Focus),
            )
            .configure_set(PickSet::Focus.run_if(PickingPluginsSettings::interaction_should_run));

        app.add_systems(
            (
                event_bubbling::<output::Over>,
                event_bubbling::<output::Out>,
                event_bubbling::<output::Down>,
                event_bubbling::<output::Up>,
                event_bubbling::<output::Click>,
                event_bubbling::<output::Move>,
                event_bubbling::<output::DragStart>,
                event_bubbling::<output::Drag>,
                event_bubbling::<output::DragEnd>,
                event_bubbling::<output::DragEnter>,
                event_bubbling::<output::DragOver>,
                event_bubbling::<output::DragLeave>,
                event_bubbling::<output::Drop>,
            )
                .in_set(PickSet::EventListeners),
        )
        .configure_set(
            PickSet::EventListeners.run_if(PickingPluginsSettings::interaction_should_run),
        );

        app.configure_sets(
            (PickSet::Input, PickSet::PostInput)
                .chain()
                .in_base_set(CoreSet::First),
        )
        .configure_sets(
            (
                PickSet::ProcessInput,
                PickSet::Focus,
                PickSet::Backend,
                PickSet::EventListeners,
            )
                .chain()
                .in_base_set(CoreSet::PreUpdate),
        );
    }
}
