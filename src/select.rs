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

/// Given the currently hovered mesh, checks for a user click and if detected, sets the selected
/// field in the entity's component to true.
pub fn select_mesh(
    // Resources
    pick_state: Res<PickState>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    // Queries
    mut query: Query<&mut SelectablePickMesh>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        // Deselect everything
        for mut selectable in &mut query.iter_mut() {
            selectable.selected = HashSet::new();
        }
        if let Some(top_list) = pick_state.top_all() {
            for (group, entity, _intersection) in top_list {
                if let Ok(mut top_mesh) = query.get_mut(*entity) {
                    top_mesh.selected.insert(*group);
                }
            }
        }
    }
}
