//! A [`bevy`] plugin for `bevy_mod_picking` that adds multiselect functionality.
//!
//! This adds the [`PointerDeselect`] and [`PointerSelect`] [`PointerEvent`]s, including support for
//! bubbling these events.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{prelude::*, utils::hashbrown::HashSet};
use bevy_picking_core::{
    events::{Click, Down, IsPointerEvent, PointerEvent},
    pointer::{InputPress, PointerButton, PointerId},
    PickSet,
};

/// Runtime settings for the `bevy_picking_selection` plugin.
#[derive(Debug, Resource)]
pub struct SelectionSettings {
    /// A pointer clicks and nothing is beneath it, should everything be deselected?
    pub click_nothing_deselect_all: bool,
    /// When true, `Ctrl` and `Shift` inputs will trigger multiselect.
    pub use_multiselect_default_inputs: bool,
}
impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            click_nothing_deselect_all: true,
            use_multiselect_default_inputs: true,
        }
    }
}

/// Adds multiselect picking support to your app.
pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionSettings>()
            .add_event::<PointerEvent<Select>>()
            .add_event::<PointerEvent<Deselect>>()
            .add_systems(
                (
                    multiselect_events.run_if(|settings: Res<SelectionSettings>| {
                        settings.use_multiselect_default_inputs
                    }),
                )
                    .chain()
                    .in_set(PickSet::ProcessInput),
            )
            .add_systems(
                (
                    send_selection_events,
                    bevy_picking_core::events::event_bubbling::<Select>,
                    bevy_picking_core::events::event_bubbling::<Deselect>,
                    update_state_from_events,
                )
                    .in_set(PickSet::PostFocus),
            );
    }
}

/// Input state that defines whether or not the multiselect button is active. This is often the
/// `Ctrl` or `Shift` keys.
#[derive(Debug, Default, Clone, Component, PartialEq, Eq, Reflect)]
pub struct PointerMultiselect {
    /// `true` if the multiselect button(s) is active.
    pub is_pressed: bool,
}

/// Tracks the current selection state of the entity.
#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct PickSelection {
    /// `true` if this entity is selected.
    pub is_selected: bool,
}

/// Fires when an entity has been selected
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Select;
impl IsPointerEvent for Select {}

/// Fires when an entity has been deselected
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Deselect;
impl IsPointerEvent for Deselect {}

/// Marker struct used to mark pickable entities for which you don't want to trigger a deselection
/// event when picked. This is useful for gizmos or other pickable UI entities.
#[derive(Component, Debug, Copy, Clone, Reflect)]
pub struct NoDeselect;

/// Unsurprising default multiselect inputs: both control and shift keys.
pub fn multiselect_events(
    keyboard: Res<Input<KeyCode>>,
    mut pointer_query: Query<&mut PointerMultiselect>,
) {
    let is_multiselect_pressed = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for mut multiselect in pointer_query.iter_mut() {
        multiselect.is_pressed = is_multiselect_pressed;
    }
}

/// Determines which entities have been selected or deselected, and sends [`PointerSelect`] and
/// [`PointerDeselect`] events corresponding to these state changes.
pub fn send_selection_events(
    settings: Res<SelectionSettings>,
    mut pointer_down: EventReader<PointerEvent<Down>>,
    mut presses: EventReader<InputPress>,
    mut pointer_click: EventReader<PointerEvent<Click>>,
    pointers: Query<(&PointerId, &PointerMultiselect)>,
    no_deselect: Query<&NoDeselect>,
    selectables: Query<(Entity, &PickSelection)>,
    // Output
    mut selections: EventWriter<PointerEvent<Select>>,
    mut deselections: EventWriter<PointerEvent<Deselect>>,
) {
    // Pointers that have clicked on something.
    let mut pointer_down_list = HashSet::new();

    for down in pointer_down.iter() {
        pointer_down_list.insert(down.pointer_id());
        let multiselect = pointers
            .iter()
            .find_map(|(id, multi)| (id == &down.pointer_id()).then_some(multi.is_pressed))
            .unwrap_or(false);
        let target_can_deselect = no_deselect.get(down.target()).is_err();
        // Deselect everything
        if !multiselect && target_can_deselect {
            for (entity, selection) in selectables.iter() {
                let not_click_target = down.target() != entity;
                if selection.is_selected && not_click_target {
                    deselections.send(PointerEvent::new(&down.pointer_id(), &entity, Deselect))
                }
            }
        }
    }

    // If a pointer has pressed, but did not press on anything, this means it clicked on nothing. If
    // so, and the setting is enabled, deselect everything.
    if settings.click_nothing_deselect_all {
        for press in presses
            .iter()
            .filter(|p| p.is_just_down(PointerButton::Primary))
        {
            let id = &press.pointer_id();
            let multiselect = pointers
                .iter()
                .find_map(|(this_id, multi)| (this_id == id).then_some(multi.is_pressed))
                .unwrap_or(false);
            if !pointer_down_list.contains(id) && !multiselect {
                for (entity, selection) in selectables.iter() {
                    if selection.is_selected {
                        deselections.send(PointerEvent::new(id, &entity, Deselect))
                    }
                }
            }
        }
    }

    for click in pointer_click.iter() {
        let multiselect = pointers
            .iter()
            .find_map(|(id, multi)| id.eq(&click.pointer_id()).then_some(multi.is_pressed))
            .unwrap_or(false);
        if let Ok((entity, selection)) = selectables.get(click.target()) {
            if multiselect {
                match selection.is_selected {
                    true => {
                        deselections.send(PointerEvent::new(&click.pointer_id(), &entity, Deselect))
                    }
                    false => {
                        selections.send(PointerEvent::new(&click.pointer_id(), &entity, Select))
                    }
                }
            } else if !selection.is_selected {
                selections.send(PointerEvent::new(&click.pointer_id(), &entity, Select))
            }
        }
    }
}

/// Update entity selection component state from pointer events.
pub fn update_state_from_events(
    mut selectables: Query<&mut PickSelection>,
    mut selections: EventReader<PointerEvent<Select>>,
    mut deselections: EventReader<PointerEvent<Deselect>>,
) {
    for selection in selections.iter() {
        if let Ok(mut select_me) = selectables.get_mut(selection.target()) {
            select_me.is_selected = true;
        }
    }
    for deselection in deselections.iter() {
        if let Ok(mut deselect_me) = selectables.get_mut(deselection.target()) {
            deselect_me.is_selected = false;
        }
    }
}
