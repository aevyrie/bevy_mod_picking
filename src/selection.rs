use crate::PickingPluginState;
use bevy::prelude::*;

/// Tracks the current selection state to be used with change tracking in the events system. Meshes
/// with [Selection] will have selection state managed.
///
/// # Requirements
///
/// An entity with the [Selection] component must also have an [Interaction] component.
#[derive(Debug, Copy, Clone)]
pub struct Selection {
    selected: bool,
}

impl Selection {
    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection { selected: false }
    }
}

pub fn mesh_selection(
    state: Res<PickingPluginState>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    keyboard_input: Res<Input<KeyCode>>,
    query_changed: Query<&Interaction, Changed<Interaction>>,
    mut query_all: Query<(&mut Selection, &Interaction)>,
    node_query: Query<&Interaction, Without<Selection>>,
) {
    if state.paused_for_ui || !state.enabled {
        return;
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
        for (mut selection, _interaction) in &mut query_all.iter_mut() {
            if !selection.selected {
                selection.selected = true;
            }
        }
    } else if new_selection {
        // Some pickable mesh has been clicked on - figure out what to select or deselect
        for (mut selection, interaction) in &mut query_all.iter_mut() {
            if selection.selected
                && *interaction != Interaction::Clicked
                && !keyboard_input.pressed(KeyCode::LControl)
            {
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
        // This branch deselects everything if the user clicks, but not on a pickable mesh or UI
        let mut ui_click = false;
        for interaction in node_query.iter() {
            // Check if anything in the UI is being interacted with
            if *interaction == Interaction::Clicked && !keyboard_input.pressed(KeyCode::LControl) {
                ui_click = true;
            }
        }
        let user_click =
            mouse_button_input.just_pressed(MouseButton::Left) || touches_input.just_released(0);
        if user_click && !ui_click {
            for (mut selection, _interaction) in &mut query_all.iter_mut() {
                if selection.selected {
                    selection.selected = false;
                }
            }
        }
    }
}
