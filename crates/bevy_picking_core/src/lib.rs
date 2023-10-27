//! Core functionality and types required for `bevy_mod_picking` to function.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

pub mod backend;
pub mod events;
pub mod focus;
pub mod pointer;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

use bevy_eventlistener::{prelude::*, EventListenerSet};
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
    /// Whether or not systems updating entities' [`PickingInteraction`](focus::PickingInteraction)
    /// component should be running.
    pub fn interaction_should_run(state: Res<Self>) -> bool {
        state.enable_interacting && state.enable
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

/// An optional component that overrides default picking behavior for an entity.
#[derive(Component, Debug, Clone, Reflect, PartialEq, Eq)]
pub struct Pickable {
    /// Should this entity block entities below it from being picked?
    ///
    /// This is useful if you want an entity to exist in the hierarchy, but want picking to "pass
    /// through" to lower layers and allow items below this one to be the target of picking events
    /// **as well as** this one.
    ///
    /// Entities without the [`Pickable`] component will block by default.
    pub should_block_lower: bool,
    /// Should this entity emit events when targeted?
    ///
    /// If this is set to `false` and `should_block_lower` is set to true, this entity will block
    /// lower entities from being interacted and at the same time will itself not emit any events.
    ///
    /// Entities without the [`Pickable`] component will emit events by default.
    pub should_emit_events: bool,
}

impl Pickable {
    /// This entity will not block entities beneath it, nor will it emit events.
    ///
    /// If a backend reports this entity as being hit, the picking plugin will completely ignore it.
    pub const IGNORE: Self = Self {
        should_block_lower: false,
        should_emit_events: false,
    };
}

impl Default for Pickable {
    fn default() -> Self {
        Self {
            should_block_lower: true,
            should_emit_events: true,
        }
    }
}

/// Components needed to build a pointer. Multiple pointers can be active at once, with each pointer
/// being an entity.
///
/// `Mouse` and `Touch` pointers are automatically spawned as needed. Use this bundle if you are
/// spawning a custom `PointerId::Custom` pointer, either for testing, as a software controlled
/// pointer, or if you are replacing the default touch and mouse inputs.
#[derive(Bundle)]
pub struct PointerCoreBundle {
    /// The pointer's unique [`PointerId`](pointer::PointerId).
    pub id: pointer::PointerId,
    /// Tracks the pointer's location.
    pub location: pointer::PointerLocation,
    /// Tracks the pointer's button press state.
    pub click: pointer::PointerPress,
    /// The interaction state of any hovered entities.
    pub interaction: pointer::PointerInteraction,
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
            interaction: pointer::PointerInteraction::default(),
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
            )
            .configure_sets(First, (PickSet::Input, PickSet::PostInput).chain())
            .configure_sets(
                PreUpdate,
                (
                    PickSet::ProcessInput,
                    PickSet::Backend,
                    PickSet::Focus.run_if(PickingPluginsSettings::interaction_should_run),
                    PickSet::PostFocus,
                    EventListenerSet,
                    PickSet::Last,
                )
                    .chain(),
            );
    }
}

/// Generates [`Pointer`](events::Pointer) events and handles event bubbling.
pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        use events::*;
        use focus::{update_focus, update_interactions};

        app.init_resource::<focus::HoverMap>()
            .init_resource::<focus::PreviousHoverMap>()
            .init_resource::<DragMap>()
            .add_event::<PointerCancel>()
            .add_systems(
                PreUpdate,
                (
                    update_focus,
                    pointer_events,
                    update_interactions,
                    send_click_and_drag_events,
                    send_drag_over_events,
                )
                    .chain()
                    .in_set(PickSet::Focus),
            )
            .add_plugins((
                EventListenerPlugin::<Pointer<Over>>::default(),
                EventListenerPlugin::<Pointer<Out>>::default(),
                EventListenerPlugin::<Pointer<Down>>::default(),
                EventListenerPlugin::<Pointer<Up>>::default(),
                EventListenerPlugin::<Pointer<Click>>::default(),
                EventListenerPlugin::<Pointer<Move>>::default(),
                EventListenerPlugin::<Pointer<DragStart>>::default(),
                EventListenerPlugin::<Pointer<Drag>>::default(),
                EventListenerPlugin::<Pointer<DragEnd>>::default(),
                EventListenerPlugin::<Pointer<DragEnter>>::default(),
                EventListenerPlugin::<Pointer<DragOver>>::default(),
                EventListenerPlugin::<Pointer<DragLeave>>::default(),
                EventListenerPlugin::<Pointer<Drop>>::default(),
            ));
    }
}
