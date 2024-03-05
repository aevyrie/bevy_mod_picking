//! A plugin for `bevy_mod_picking` that adds multiselect functionality.
//!
//! This adds the [`Deselect`] and [`Select`] [`Pointer`] events, including support for bubbling
//! these events.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, ButtonInput};
use bevy_reflect::prelude::*;
use bevy_utils::hashbrown::HashSet;

use bevy_eventlistener::prelude::*;

use bevy_picking_core::{
    events::{Click, Down, Pointer},
    pointer::{InputPress, PointerButton, PointerId, PointerLocation},
    PickSet, PickingPluginsSettings,
};

/// Runtime settings for the `bevy_picking_selection` plugin.
#[derive(Debug, Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct SelectionPluginSettings {
    /// Should selection systems run?
    pub is_enabled: bool,
    /// A pointer clicks and nothing is beneath it, should everything be deselected?
    pub click_nothing_deselect_all: bool,
    /// When true, `Ctrl` and `Shift` inputs will trigger multiselect.
    pub use_multiselect_default_inputs: bool,
}

impl SelectionPluginSettings {
    /// Whether or not selection systems should run
    pub fn should_run(settings: Res<Self>, main_settings: Res<PickingPluginsSettings>) -> bool {
        settings.is_enabled && main_settings.is_enabled
    }

    /// Whether or not multiselect input systems should run
    pub fn multiselect_should_run(
        settings: Res<Self>,
        main_settings: Res<PickingPluginsSettings>,
    ) -> bool {
        settings.use_multiselect_default_inputs && settings.is_enabled && main_settings.is_enabled
    }
}

impl Default for SelectionPluginSettings {
    fn default() -> Self {
        Self {
            is_enabled: true,
            click_nothing_deselect_all: true,
            use_multiselect_default_inputs: true,
        }
    }
}

/// Adds multiselect picking support to your app.
pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionPluginSettings>()
            .add_event::<Pointer<Select>>()
            .add_event::<Pointer<Deselect>>()
            .add_plugins((
                EventListenerPlugin::<Pointer<Select>>::default(),
                EventListenerPlugin::<Pointer<Deselect>>::default(),
            ))
            .add_systems(
                PreUpdate,
                (
                    (multiselect_events)
                        .chain()
                        .in_set(PickSet::ProcessInput)
                        .run_if(SelectionPluginSettings::multiselect_should_run),
                    (send_selection_events, update_state_from_events)
                        .chain()
                        .in_set(PickSet::PostFocus)
                        .run_if(SelectionPluginSettings::should_run),
                ),
            )
            .register_type::<SelectionPluginSettings>()
            .register_type::<PointerMultiselect>()
            .register_type::<PickSelection>()
            .register_type::<NoDeselect>();
    }
}

/// A component for pointers that defines whether or not the multiselect button is active. This is
/// often the `Ctrl` or `Shift` keys.
#[derive(Debug, Default, Clone, Component, PartialEq, Eq, Reflect)]
#[reflect(Component, Default)]
pub struct PointerMultiselect {
    /// `true` if the multiselect button(s) is active.
    pub is_pressed: bool,
}

/// Tracks the current selection state of the entity.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct PickSelection {
    /// `true` if this entity is selected.
    pub is_selected: bool,
}

/// Marker struct used to mark pickable entities for which you don't want to trigger a deselection
/// event when picked. This is useful for gizmos or other pickable UI entities.
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct NoDeselect;

/// Fires when an entity has been selected
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Select;

/// Fires when an entity has been deselected
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Deselect;

/// Unsurprising default multiselect inputs: both control and shift keys.
pub fn multiselect_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pointer_query: Query<&mut PointerMultiselect>,
) {
    let is_multiselect_pressed = keyboard.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ]);

    for mut multiselect in pointer_query.iter_mut() {
        multiselect.is_pressed = is_multiselect_pressed;
    }
}

/// Determines which entities have been selected or deselected, and sends [`Select`] and
/// [`Deselect`] events corresponding to these state changes.
pub fn send_selection_events(
    settings: Res<SelectionPluginSettings>,
    mut pointer_down: EventReader<Pointer<Down>>,
    mut presses: EventReader<InputPress>,
    mut pointer_click: EventReader<Pointer<Click>>,
    pointers: Query<(&PointerId, &PointerMultiselect, &PointerLocation)>,
    no_deselect: Query<&NoDeselect>,
    selectables: Query<(Entity, &PickSelection)>,
    // Output
    mut selections: EventWriter<Pointer<Select>>,
    mut deselections: EventWriter<Pointer<Deselect>>,
) {
    // Pointers that have clicked on something.
    let mut pointer_down_list = HashSet::new();

    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: _,
    } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        pointer_down_list.insert(pointer_id);
        let multiselect = pointers
            .iter()
            .find_map(|(id, multi, _)| (id == pointer_id).then_some(multi.is_pressed))
            .unwrap_or(false);
        let target_can_deselect = no_deselect.get(*target).is_err();
        // Deselect everything
        if !multiselect && target_can_deselect {
            for (entity, selection) in selectables.iter() {
                let not_click_target = *target != entity;
                if selection.is_selected && not_click_target {
                    deselections.send(Pointer::new(
                        *pointer_id,
                        pointer_location.to_owned(),
                        entity,
                        Deselect,
                    ));
                }
            }
        }
    }

    // If a pointer has pressed, but did not press on anything, this means it clicked on nothing. If
    // so, and the setting is enabled, deselect everything.
    if settings.click_nothing_deselect_all {
        for press in presses
            .read()
            .filter(|p| p.is_just_down(PointerButton::Primary))
        {
            let id = press.pointer_id;
            let Some((multiselect, location)) =
                pointers.iter().find_map(|(this_id, multi, location)| {
                    (*this_id == id)
                        .then_some(location.location.clone())
                        .flatten()
                        .map(|location| (multi.is_pressed, location))
                })
            else {
                continue;
            };
            if !pointer_down_list.contains(&id) && !multiselect {
                for (entity, selection) in selectables.iter() {
                    if selection.is_selected {
                        deselections.send(Pointer::new(id, location.clone(), entity, Deselect));
                    }
                }
            }
        }
    }

    for Pointer {
        pointer_id,
        pointer_location,
        target,
        event: _,
    } in pointer_click
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        let multiselect = pointers
            .iter()
            .find_map(|(id, multi, _)| id.eq(pointer_id).then_some(multi.is_pressed))
            .unwrap_or(false);
        if let Ok((entity, selection)) = selectables.get(*target) {
            if multiselect {
                match selection.is_selected {
                    true => {
                        deselections.send(Pointer::new(
                            *pointer_id,
                            pointer_location.to_owned(),
                            entity,
                            Deselect,
                        ));
                    }
                    false => {
                        selections.send(Pointer::new(
                            *pointer_id,
                            pointer_location.to_owned(),
                            entity,
                            Select,
                        ));
                    }
                };
            } else if !selection.is_selected {
                selections.send(Pointer::new(
                    *pointer_id,
                    pointer_location.to_owned(),
                    entity,
                    Select,
                ));
            }
        }
    }
}

/// Update entity selection component state from pointer events.
pub fn update_state_from_events(
    mut selectables: Query<&mut PickSelection>,
    mut selections: EventReader<Pointer<Select>>,
    mut deselections: EventReader<Pointer<Deselect>>,
) {
    for selection in selections.read() {
        if let Ok(mut select_me) = selectables.get_mut(selection.target) {
            select_me.is_selected = true;
        }
    }
    for deselection in deselections.read() {
        if let Ok(mut deselect_me) = selectables.get_mut(deselection.target) {
            deselect_me.is_selected = false;
        }
    }
}
