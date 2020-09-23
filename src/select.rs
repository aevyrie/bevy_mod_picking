use super::*;
use bevy::prelude::*;

/// Meshes with `SelectableMesh` will have selection state managed
#[derive(Debug)]
pub struct SelectablePickMesh {
    selected: bool,
}

impl SelectablePickMesh {
    pub fn new() -> Self {
        SelectablePickMesh::default()
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl Default for SelectablePickMesh {
    fn default() -> Self {
        SelectablePickMesh { selected: false }
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
        for mut selectable in &mut query.iter() {
            selectable.selected = false;
        }

        if let Some(pick_depth) = pick_state.top(PickingGroup::default()) {
            if let Ok(mut top_mesh) = query.get_mut::<SelectablePickMesh>(pick_depth.entity) {
                top_mesh.selected = true;
            }
        }
    }
}
