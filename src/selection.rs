use crate::PausedForBlockers;
use bevy::prelude::*;

/// Tracks the current selection state to be used with change tracking in the events system. Meshes
/// with [Selection] will have selection state managed.
///
/// # Requirements
///
/// An entity with the [Selection] component must also have an [Interaction] component.
#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Selection {
    selected: bool,
}
impl Selection {
    pub fn selected(&self) -> bool {
        self.selected
    }
    /// Set the selection state.
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

/// Marker struct used to mark pickable entities for which you don't want to trigger a deselection event when picked. This is useful for gizmos or other pickable UI entities.
#[derive(Component, Debug, Copy, Clone)]
pub struct NoDeselect;

#[allow(clippy::too_many_arguments)]
pub fn mesh_selection(
    paused: Option<Res<PausedForBlockers>>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    keyboard_input: Res<Input<KeyCode>>,
    query_changed: Query<&Interaction, (Changed<Interaction>, Without<NoDeselect>)>,
    mut query_all: Query<(&mut Selection, &Interaction)>,
    node_query: Query<&Interaction, With<Node>>,
    no_deselect_query: Query<&Interaction, With<NoDeselect>>,
) {
    if let Some(paused) = paused {
        if paused.0 {
            return;
        }
    }

    // Check if something has been clicked on
    let mut new_selection = false;
    for interaction in query_changed.iter() {
        if *interaction == Interaction::Clicked {
            new_selection = true;
        }
    }

    if keyboard_input.pressed(KeyCode::LControl) && keyboard_input.pressed(KeyCode::A) {
        // The user has hit ctrl+a, select all the things!
        query_all.for_each_mut(|(mut selection, _)| {
            if !selection.selected {
                selection.selected = true;
            }
        });
    } else if new_selection {
        // Some pickable mesh has been clicked on - figure out what to select or deselect
        for (mut selection, interaction) in &mut query_all.iter_mut() {
            if selection.selected
                && *interaction != Interaction::Clicked
                && !keyboard_input.pressed(KeyCode::LControl)
            {
                // In this case, the entity is currently marked as selected, but it was not clicked
                // on (interaction), and lctrl was not being held, so it should be deselected.
                selection.selected = false;
            } else if *interaction == Interaction::Clicked
                && keyboard_input.pressed(KeyCode::LControl)
            {
                selection.selected = !selection.selected
            } else if !selection.selected && *interaction == Interaction::Clicked {
                selection.selected = true;
            }
        }
    } else {
        // This branch deselects everything if the user clicks, in empty space. Deselection is not
        // run if the UI or an item tagged with `NoDeselect` was clicked on.
        let mut ui_not_clicked = true;
        for interaction in node_query.iter() {
            // Check if anything in the UI is being interacted with
            if *interaction == Interaction::Clicked && !keyboard_input.pressed(KeyCode::LControl) {
                ui_not_clicked = false;
            }
        }
        let mut no_deselect_not_clicked = true;
        for interaction in no_deselect_query.iter() {
            if *interaction == Interaction::Clicked && !keyboard_input.pressed(KeyCode::LControl) {
                no_deselect_not_clicked = false;
            }
        }
        let mouse_clicked = mouse_button_input.just_pressed(MouseButton::Left)
            || touches_input.iter_just_pressed().next().is_some();
        if mouse_clicked && ui_not_clicked && no_deselect_not_clicked {
            for (mut selection, _interaction) in &mut query_all.iter_mut() {
                if selection.selected {
                    selection.selected = false;
                }
            }
        }
    }
}
