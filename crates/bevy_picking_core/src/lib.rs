//! Core functionality and types required for `bevy_mod_picking` to function.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

pub mod backend;
pub mod debug;
pub mod event_listening;
pub mod events;
pub mod focus;
pub mod pointer;

use bevy::prelude::*;
use focus::{interactions_from_events, update_focus};

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

/// Used to mark entities that should be pickable.
#[derive(Component, Debug, Default, Clone, Reflect)]
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
    pub interaction: focus::PointerInteraction,
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
            interaction: focus::PointerInteraction::default(),
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
    /// Reads inputs and produces [`backend::PointerHits`]s.
    Backend,
    /// Reads [`backend::PointerHits`]s, and updates focus, selection, and highlighting
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
            .add_event::<backend::PointerHits>()
            .add_systems(
                PreUpdate,
                (
                    pointer::update_pointer_map,
                    pointer::InputMove::receive,
                    pointer::InputPress::receive,
                )
                    .in_set(PickSet::ProcessInput),
            );
    }
}

/// Generates [`PointerEvent`](events::PointerEvent)s and handles event bubbling.
pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        use event_listening::*;
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
                PreUpdate,
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
            .configure_set(
                PreUpdate,
                PickSet::Focus.run_if(PickingPluginsSettings::interaction_should_run),
            );

        app.add_plugin(EventListenerPlugin::<Over>::default())
            .add_plugin(EventListenerPlugin::<Out>::default())
            .add_plugin(EventListenerPlugin::<Down>::default())
            .add_plugin(EventListenerPlugin::<Up>::default())
            .add_plugin(EventListenerPlugin::<Click>::default())
            .add_plugin(EventListenerPlugin::<Move>::default())
            .add_plugin(EventListenerPlugin::<DragStart>::default())
            .add_plugin(EventListenerPlugin::<Drag>::default())
            .add_plugin(EventListenerPlugin::<DragEnd>::default())
            .add_plugin(EventListenerPlugin::<DragEnter>::default())
            .add_plugin(EventListenerPlugin::<DragOver>::default())
            .add_plugin(EventListenerPlugin::<DragLeave>::default())
            .add_plugin(EventListenerPlugin::<Drop>::default())
            .configure_set(
                PreUpdate,
                PickSet::EventListeners.run_if(PickingPluginsSettings::interaction_should_run),
            );

        app.configure_sets(First, (PickSet::Input, PickSet::PostInput).chain())
            .configure_sets(
                PreUpdate,
                (
                    PickSet::ProcessInput,
                    PickSet::Backend,
                    PickSet::Focus,
                    PickSet::EventListeners,
                    PickSet::Last,
                )
                    .chain(),
            );
    }
}
