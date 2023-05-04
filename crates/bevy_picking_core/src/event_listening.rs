//! Event listening and bubbling.

use crate::events::{IsPointerEvent, PointerEvent};
use crate::{
    pointer::{Location, PointerId},
    PickSet,
};
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
    utils::HashMap,
};
use std::marker::PhantomData;

/// Adds event listening and bubbling support for event `E`.
pub struct EventListenerPlugin<E>(PhantomData<E>);

impl<E> Default for EventListenerPlugin<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: IsPointerEvent> Plugin for EventListenerPlugin<E> {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventCallbackGraph::<E>::default())
            .add_systems(
                PreUpdate,
                (
                    EventCallbackGraph::<E>::build.run_if(on_event::<PointerEvent<E>>()),
                    event_bubbling::<E>.run_if(on_event::<PointerEvent<E>>()),
                )
                    .chain()
                    .in_set(PickSet::EventListeners),
            );
    }
}

pub(crate) enum CallbackSystem<E: IsPointerEvent> {
    Empty,
    New(Box<dyn System<In = ListenedEvent<E>, Out = Bubble>>),
    Initialized(Box<dyn System<In = ListenedEvent<E>, Out = Bubble>>),
}

impl<E: IsPointerEvent> CallbackSystem<E> {
    pub(crate) fn is_initialized(&self) -> bool {
        matches!(self, CallbackSystem::Initialized(_))
    }

    pub(crate) fn run(&mut self, world: &mut World, event_data: ListenedEvent<E>) -> Bubble {
        if !self.is_initialized() {
            let mut temp = CallbackSystem::Empty;
            std::mem::swap(self, &mut temp);
            if let CallbackSystem::New(mut system) = temp {
                system.initialize(world);
                *self = CallbackSystem::Initialized(system);
            }
        }
        match self {
            CallbackSystem::Initialized(system) => {
                let result = system.run(event_data, world);
                system.apply_buffers(world);
                result
            }
            _ => unreachable!(),
        }
    }
}

/// Used to attach a callback to an entity. This callback is executed any time a pointer event
/// triggers this listener when bubbling up the entity hierarchy.
///
/// Callback systems will give you access to the `ListenedEvent` that triggered the event listener.
/// This includes the `listener` which is the entity with the `OnPointer` component, and the
/// `target` which is the entity that generated the event. The `target` can be this entity or any of
/// its children.
#[derive(Component, Reflect)]
pub struct OnPointer<E: IsPointerEvent> {
    #[reflect(ignore)]
    /// A function that is called when the event listener is triggered.
    pub(crate) callback: CallbackSystem<E>,
}

impl<E: IsPointerEvent> OnPointer<E> {
    /// Run a callback system any time this event listener is triggered.
    pub fn run_callback<M>(callback: impl IntoSystem<ListenedEvent<E>, Bubble, M>) -> Self {
        Self {
            callback: CallbackSystem::New(Box::new(IntoSystem::into_system(callback))),
        }
    }

    /// Add a single [`Command`] any time this event listener is triggered. The command must
    /// implement `From<ListenedEvent<E>>`.
    pub fn add_command<C: From<ListenedEvent<E>> + Command + Send + Sync + 'static>() -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut commands: Commands| {
                commands.add(C::from(event));
                Bubble::Up
            },
        )
    }

    /// Get mutable access to [`Commands`] any time this event listener is triggered.
    pub fn commands_mut(func: fn(&ListenedEvent<E>, &mut Commands)) -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut commands: Commands| {
                func(&event, &mut commands);
                Bubble::Up
            },
        )
    }

    /// Get mutable access to the target entity's [`EntityCommands`] using a closure any time this
    /// event listener is triggered.
    pub fn target_commands_mut(func: fn(&ListenedEvent<E>, &mut EntityCommands)) -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut commands: Commands| {
                func(&event, &mut commands.entity(event.target));
                Bubble::Up
            },
        )
    }

    /// Insert a bundle on the target entity any time this event listener is triggered.
    pub fn target_insert(bundle: impl Bundle + Clone) -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut commands: Commands| {
                let bundle = bundle.clone();
                commands.entity(event.target).insert(bundle);
                Bubble::Up
            },
        )
    }

    /// Remove a bundle from the target entity any time this event listener is triggered.
    pub fn target_remove<B: Bundle>() -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut commands: Commands| {
                commands.entity(event.target).remove::<B>();
                Bubble::Up
            },
        )
    }

    /// Get mutable access to a specific component on the target entity using a closure any time
    /// this event listener is triggered. If the component does not exist, an error will be logged.
    pub fn target_component_mut<C: Component>(func: fn(&ListenedEvent<E>, &mut C)) -> Self {
        Self::run_callback(
            move |In(event): In<ListenedEvent<E>>, mut query: Query<&mut C>| {
                if let Ok(mut component) = query.get_mut(event.target) {
                    func(&event, &mut component);
                } else {
                    error!("Component {:?} not found on entity {:?} during pointer callback for event {:?}", std::any::type_name::<C>(), event.target, std::any::type_name::<E>());
                }
                Bubble::Up
            },
        )
    }

    /// Send an event `F`  any time this event listener is triggered. `F` must implement
    /// `From<ListenedEvent<E>>`.
    pub fn send_event<F: Event + From<ListenedEvent<E>>>() -> Self {
        Self::run_callback(|In(event): In<ListenedEvent<E>>, mut ev: EventWriter<F>| {
            ev.send(F::from(event));
            Bubble::Up
        })
    }

    /// Take the boxed system callback out of this listener, leaving an empty one behind.
    pub(crate) fn take(&mut self) -> CallbackSystem<E> {
        let mut temp = CallbackSystem::Empty;
        std::mem::swap(&mut self.callback, &mut temp);
        temp
    }
}

/// Data from a pointer event returned by an [`OnPointer`].
///
/// This is similar to the [`PointerEvent`] struct, with the addition of event listener data.
#[derive(Clone, PartialEq, Debug)]
pub struct ListenedEvent<E: IsPointerEvent> {
    /// The pointer involved in this event.
    pub pointer_id: PointerId,
    /// The location of the pointer during this event
    pub pointer_location: Location,
    /// The entity that was listening for this event.
    pub listener: Entity,
    /// The entity that this event was originally triggered on.
    pub target: Entity,
    /// Event-specific information, e.g. data specific to `Drag` events, that isn't shared amongst
    /// all pointer events. This contains the [`HitData`](crate::backend::HitData) if it is
    /// available.
    pub pointer_event: E,
}

impl<E: IsPointerEvent> std::ops::Deref for ListenedEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.pointer_event
    }
}

/// Determines whether an event should continue to bubble up the entity hierarchy.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum Bubble {
    /// Allows this event to bubble up to its parent.
    #[default]
    Up,
    /// Stops this event from bubbling to the next parent.
    Burst,
}

impl Bubble {
    /// Stop this event from bubbling to the next parent.
    pub fn burst(&mut self) {
        *self = Bubble::Burst;
    }
}

/// In order to traverse the entity hierarchy and read events, we need to extract the callbacks out
/// of their components before they can be run. This is because running callbacks requires mutable
/// access to the [`World`], which we can't do if we are also trying to mutate the [`OnPointer`]'s
/// inner callback state.
#[derive(Resource)]
pub struct EventCallbackGraph<E: IsPointerEvent> {
    /// All the events of type `E` that were emitted this frame, and encountered an [`OnPointer<E>`]
    /// while traversing the entity hierarchy. The `Entity` in the tuple is the root node to use
    /// when traversing the listener graph.
    pub(crate) events: Vec<(PointerEvent<E>, Entity)>,
    /// Traversing the entity hierarchy for each event can visit the same entity multiple times.
    /// Storing the callbacks for each of these potentially visited entities in a graph structure is
    /// necessary for a few reasons:
    ///
    /// - Callback systems cannot implement `Clone`, so we can only have one copy of each callback
    ///   system.
    /// - For complex hierarchies, this is more memory efficient.
    /// - This allows us to jump to the next listener in the hierarchy without unnecessary
    ///   traversal. When bubbling many events of the same type `E` through the same entity tree,
    ///   this can save a significant amount of work.
    pub(crate) listener_graph: HashMap<Entity, (CallbackSystem<E>, Option<Entity>)>,
}

impl<E: IsPointerEvent> EventCallbackGraph<E> {
    pub(crate) fn build(
        mut events: EventReader<PointerEvent<E>>,
        mut listeners: Query<(Option<&mut OnPointer<E>>, Option<&Parent>)>,
        mut callbacks: ResMut<EventCallbackGraph<E>>,
    ) {
        let mut filtered_events = Vec::new();
        let mut listener_map = HashMap::new();

        for event in events.iter() {
            let mut this_node = event.target;
            let mut prev_node = this_node;
            let mut root_node = None;

            'bubble_traversal: loop {
                if let Some((_, next_node)) = listener_map.get(&this_node) {
                    if root_node.is_none() {
                        root_node = Some(this_node);
                    }
                    // If the current entity is already in the map, use it to jump ahead
                    match next_node {
                        Some(next_node) => this_node = *next_node,
                        None => break 'bubble_traversal, // Bubble reached the surface!
                    }
                } else if let Ok((event_listener, parent)) = listeners.get_mut(this_node) {
                    // Otherwise, get the current entity's data with a query
                    if let Some(mut event_listener) = event_listener {
                        // If it has an event listener, we need to add it to the map
                        listener_map.insert(this_node, (event_listener.take(), None));
                        if let Some((_, prev_nodes_next_node)) = listener_map.get_mut(&prev_node) {
                            if prev_node != this_node {
                                *prev_nodes_next_node = Some(this_node);
                            }
                        }
                        if root_node.is_none() {
                            root_node = Some(this_node);
                        }
                        prev_node = this_node;
                    }
                    match parent {
                        Some(parent) => this_node = **parent,
                        None => break 'bubble_traversal, // Bubble reached the surface!
                    }
                } else {
                    // This can be reached if the entity targeted by the event was deleted before
                    // the bubbling system could run.
                    break 'bubble_traversal;
                }
            }
            if let Some(root_node) = root_node {
                // Only add events if they interact with an event listener.
                filtered_events.push((event.to_owned(), root_node));
            }
        }
        callbacks.listener_graph = listener_map;
        callbacks.events = filtered_events;
    }
}

impl<E: IsPointerEvent> Default for EventCallbackGraph<E> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            listener_graph: HashMap::new(),
        }
    }
}

/// Bubbles [`PointerEvent`]s of event type `E`.
///
/// Event bubbling makes it simple for specific entities to listen for specific events. When a
/// `PointerEvent` event is fired, `event_bubbling` will look for an `OnPointer` on the event's
/// target entity, then walk up the hierarchy of the entity's ancestors, until a [`Bubble`]`::Pop`
/// is found or the root of the hierarchy is reached.
///
/// For every entity in the hierarchy, this system will look for an [`OnPointer`] matching the event
/// type `E`, and run the `callback` function in the event listener.
pub fn event_bubbling<E: IsPointerEvent + 'static>(world: &mut World) {
    let Some(mut callbacks) = world.remove_resource::<EventCallbackGraph<E>>() else {
        return
    };
    world.insert_resource(EventCallbackGraph::<E>::default());

    for (event, root_node) in callbacks.events.iter() {
        let mut this_node = *root_node;
        'bubble_traversal: while let Some((callback, next_node)) =
            callbacks.listener_graph.get_mut(&this_node)
        {
            let event_data = ListenedEvent {
                pointer_id: event.pointer_id,
                pointer_location: event.pointer_location.clone(),
                listener: this_node,
                target: event.target,
                pointer_event: event.event.clone(),
            };
            let callback_result = callback.run(world, event_data);
            if callback_result == Bubble::Burst {
                break 'bubble_traversal;
            }
            match next_node {
                Some(next_node) => this_node = *next_node,
                _ => break 'bubble_traversal,
            }
        }
    }

    let mut listeners = world.query::<&mut OnPointer<E>>();

    for (entity, (callback, _)) in callbacks.listener_graph.drain() {
        if let Ok(mut listener) = listeners.get_mut(world, entity) {
            listener.callback = callback;
        }
    }
}
