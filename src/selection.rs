use bevy::prelude::*;

/// Meshes with `SelectableMesh` will have selection state managed
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
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    keyboard_input: Res<Input<KeyCode>>,
    query_changed: Query<&Interaction, Changed<Interaction>>,
    mut query_all: Query<(&mut Selection, &Interaction)>,
    node_query: Query<&Interaction, With<Node>>,
) {
    let mut new_selection = false;
    for interaction in query_changed.iter() {
        if *interaction == Interaction::Clicked {
            new_selection = true;
        }
    }

    if new_selection {
        // Unselect everything else
        // TODO multi select would check if ctrl or shift is being held before clearing
        for (mut selection, interaction) in &mut query_all.iter_mut() {
            if selection.selected && *interaction != Interaction::Clicked && !keyboard_input.pressed(KeyCode::LControl) {
                selection.selected = false;
            }
            if *interaction == Interaction::Clicked {
                selection.selected = true;
            }
        }
    } else {
        let mut ui_click = false;
        // If anyting in the UI is being interacted with, set all pick interactions to none and exit
        for interaction in node_query.iter() {
            if *interaction == Interaction::Clicked {
                ui_click = true;
            }
        }
        let user_click =
            mouse_button_input.just_pressed(MouseButton::Left) || touches_input.just_released(0);

        // If the user clicked, but not on the ui, deslect everything
        if user_click && !ui_click {
            for (mut selection, _interaction) in &mut query_all.iter_mut() {
                if selection.selected {
                    selection.selected = false;
                }
            }
        }
    }
}
