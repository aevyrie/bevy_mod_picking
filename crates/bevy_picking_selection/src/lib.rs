//! A [`bevy`] plugin for `bevy_mod_picking` that adds multiselect functionality.
//!
//! This adds the [`PointerDeselect`] and [`PointerSelect`] [`PointerEvent`]s, including support for
//! bubbling these events.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_picking_core::{
    output::{IsPointerEvent, PointerEvent},
    pointer::PointerId,
    PickStage,
};

/// Adds multiselect picking support to your app.
pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointerSelect>()
            .add_event::<PointerDeselect>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Focus)
                    .before(PickStage::EventListeners)
                    .with_system(multiselect_events)
                    .with_system(send_selection_events.after(multiselect_events))
                    .with_system(update_state_from_events.after(send_selection_events)),
            )
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .after(PickStage::Focus)
                    .label(PickStage::EventListeners)
                    .with_system(bevy_picking_core::output::event_bubbling::<Select>)
                    .with_system(bevy_picking_core::output::event_bubbling::<Deselect>),
            );
    }
}

/// Input state that defines whether or not the multiselect button is active. This is often the
/// `Ctrl` or `Shift` keys.
#[derive(Debug, Default, Clone, Component, PartialEq, Eq)]
pub struct PointerMultiselect {
    /// `true` if the multiselect button(s) is active.
    pub is_pressed: bool,
}

/// Tracks the current selection state of the entity.
#[derive(Component, Debug, Default, Clone)]
pub struct PickSelection {
    /// `true` if this entity is selected.
    pub is_selected: bool,
}

/// Fires when an entity has been selected
pub type PointerSelect = PointerEvent<Select>;
/// The inner [`PointerEvent`] type for [`PointerSelect`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Select;

/// Fires when an entity has been deselected
pub type PointerDeselect = PointerEvent<Deselect>;
/// The inner [`PointerEvent`] type for [`PointerDeselect`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Reflect)]
pub struct Deselect;

/// Marker struct used to mark pickable entities for which you don't want to trigger a deselection
/// event when picked. This is useful for gizmos or other pickable UI entities.
#[derive(Component, Debug, Copy, Clone)]
pub struct NoDeselect;

/// Unsurprising default multiselect inputs: both  control and shift keys.
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
    mut pointer_down: EventReader<bevy_picking_core::output::PointerDown>,
    mut pointer_click: EventReader<bevy_picking_core::output::PointerClick>,
    pointers: Query<(&PointerId, &PointerMultiselect)>,
    no_deselect: Query<&NoDeselect>,
    selectables: Query<(Entity, &PickSelection)>,
    // Output
    mut selections: EventWriter<PointerSelect>,
    mut deselections: EventWriter<PointerDeselect>,
) {
    for down in pointer_down.iter() {
        let multiselect = pointers
            .iter()
            .find_map(|(id, multi)| id.eq(&down.id()).then_some(multi.is_pressed))
            .unwrap_or(false);
        let target_should_deselect = no_deselect.get(down.target()).is_err();
        // Deselect everything
        if !multiselect && target_should_deselect {
            for (entity, selection) in selectables.iter() {
                let not_click_target = down.target() != entity;
                if selection.is_selected && not_click_target {
                    deselections.send(PointerDeselect::new(&down.id(), &entity, Deselect))
                }
            }
        }
    }

    for click in pointer_click.iter() {
        let multiselect = pointers
            .iter()
            .find_map(|(id, multi)| id.eq(&click.id()).then_some(multi.is_pressed))
            .unwrap_or(false);
        if let Ok((entity, selection)) = selectables.get(click.target()) {
            if multiselect {
                match selection.is_selected {
                    true => deselections.send(PointerDeselect::new(&click.id(), &entity, Deselect)),
                    false => selections.send(PointerSelect::new(&click.id(), &entity, Select)),
                }
            } else if !selection.is_selected {
                selections.send(PointerSelect::new(&click.id(), &entity, Select))
            }
        }
    }
}

/// Update entity selection component state from pointer events.
pub fn update_state_from_events(
    mut selectables: Query<&mut PickSelection>,
    mut selections: EventReader<PointerSelect>,
    mut deselections: EventReader<PointerDeselect>,
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
