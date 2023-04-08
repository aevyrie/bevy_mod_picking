//! Core functionality and types required for `bevy_mod_picking` to function.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

pub mod backend;
pub mod debug;
pub mod events;
pub mod focus;
pub mod pointer;

use bevy::prelude::*;
use focus::update_focus;

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

/// A component that marks an entity as pickable.
#[derive(Clone, Copy, Debug, Default, Component, Reflect)]
pub struct Pickable;

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
    pub interaction: events::PointerInteraction,
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
            interaction: events::PointerInteraction::default(),
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
    /// Runs after all the focus systems are done, before event listeners are triggered.
    PostFocus,
    /// Updates event listeners and bubbles [`events::PointerEvent`]s
    EventListeners,
    /// Runs after all other sets
    Last,
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

/// Generates [`PointerEvent`](events::PointerEvent)s and handles event bubbling.
pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        use events::*;

        app.init_resource::<focus::HoverMap>()
            .init_resource::<focus::PreviousHoverMap>()
            .init_resource::<DragMap>()
            .add_event::<PointerCancel>()
            .add_event::<PointerEvent<Over>>()
            .add_event::<PointerEvent<Out>>()
            .add_event::<PointerEvent<Down>>()
            .add_event::<PointerEvent<Up>>()
            .add_event::<PointerEvent<Click>>()
            .add_event::<PointerEvent<Move>>()
            .add_event::<PointerEvent<DragStart>>()
            .add_event::<PointerEvent<Drag>>()
            .add_event::<PointerEvent<DragEnd>>()
            .add_event::<PointerEvent<DragEnter>>()
            .add_event::<PointerEvent<DragOver>>()
            .add_event::<PointerEvent<DragLeave>>()
            .add_event::<PointerEvent<Drop>>()
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
                event_bubbling::<Over>.run_if(on_event::<PointerEvent<Over>>()),
                event_bubbling::<Out>.run_if(on_event::<PointerEvent<Out>>()),
                event_bubbling::<Down>.run_if(on_event::<PointerEvent<Down>>()),
                event_bubbling::<Up>.run_if(on_event::<PointerEvent<Up>>()),
                event_bubbling::<Click>.run_if(on_event::<PointerEvent<Click>>()),
                event_bubbling::<Move>.run_if(on_event::<PointerEvent<Move>>()),
                event_bubbling::<DragStart>.run_if(on_event::<PointerEvent<DragStart>>()),
                event_bubbling::<Drag>.run_if(on_event::<PointerEvent<Drag>>()),
                event_bubbling::<DragEnd>.run_if(on_event::<PointerEvent<DragEnd>>()),
                event_bubbling::<DragEnter>.run_if(on_event::<PointerEvent<DragEnter>>()),
                event_bubbling::<DragOver>.run_if(on_event::<PointerEvent<DragOver>>()),
                event_bubbling::<DragLeave>.run_if(on_event::<PointerEvent<DragLeave>>()),
                event_bubbling::<Drop>.run_if(on_event::<PointerEvent<Drop>>()),
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
                PickSet::Backend,
                PickSet::Focus,
                PickSet::EventListeners,
                PickSet::Last,
            )
                .chain()
                .in_base_set(CoreSet::PreUpdate),
        );
    }
}
