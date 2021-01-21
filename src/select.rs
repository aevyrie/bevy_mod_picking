use super::*;
use bevy::prelude::*;

/// Meshes with `SelectableMesh` will have selection state managed
#[derive(Debug)]
pub struct SelectablePickMesh {
    selected: bool,
}

impl SelectablePickMesh {
    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl Default for SelectablePickMesh {
    fn default() -> Self {
        SelectablePickMesh {
            selected: false,
        }
    }
}

/// Update all entities with the groups they are selected in.
pub fn select_mesh(
    // Resources
    mouse_button_inputs: Res<Input<MouseButton>>,
    // Queries
    mut query: Query<(&mut SelectablePickMesh, &InteractableMesh)>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        // Update Selections
        for (mut selectable, interactable) in &mut query.iter_mut() {
            selectable.selected = interactable.just_pressed(MouseButton::Left);
        }
    }
}
