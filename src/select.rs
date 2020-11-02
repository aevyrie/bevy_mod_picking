use super::*;
use bevy::prelude::*;
use std::collections::HashSet;

/// Meshes with `SelectableMesh` will have selection state managed
#[derive(Debug)]
pub struct SelectablePickMesh {
    selected: HashSet<Group>,
}

impl SelectablePickMesh {
    pub fn new() -> Self {
        SelectablePickMesh::default()
    }
    pub fn selected(&self, group: &Group) -> bool {
        self.selected.get(group).is_some()
    }
}

impl Default for SelectablePickMesh {
    fn default() -> Self {
        SelectablePickMesh {
            selected: HashSet::new(),
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
            selectable.selected = interactable.groups_just_pressed(MouseButton::Left);
        }
    }
}
