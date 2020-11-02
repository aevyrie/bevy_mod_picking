use super::{select::*, *};
use bevy::{prelude::*, render::color::Color};

#[derive(Debug)]
pub struct PickHighlightParams {
    hover_color: Color,
    selection_color: Color,
}

impl PickHighlightParams {
    pub fn hover_color_mut(&mut self) -> &mut Color {
        &mut self.hover_color
    }
    pub fn selection_color_mut(&mut self) -> &mut Color {
        &mut self.selection_color
    }
    pub fn set_hover_color(&mut self, color: Color) {
        self.hover_color = color;
    }
    pub fn set_selection_color(&mut self, color: Color) {
        self.selection_color = color;
    }
}

impl Default for PickHighlightParams {
    fn default() -> Self {
        PickHighlightParams {
            hover_color: Color::rgb(0.3, 0.5, 0.8),
            selection_color: Color::rgb(0.3, 0.8, 0.5),
        }
    }
}

/// Meshes with `HighlightablePickMesh` will be highlighted when hovered over.
/// If the mesh also has the `SelectablePickMesh` component, it will highlight when selected.
#[derive(Debug)]
pub struct HighlightablePickMesh {
    // Stores the initial color of the mesh material prior to selecting/hovering
    initial_color: Option<Color>,
}

impl HighlightablePickMesh {
    pub fn new() -> Self {
        HighlightablePickMesh::default()
    }
}

impl Default for HighlightablePickMesh {
    fn default() -> Self {
        HighlightablePickMesh {
            initial_color: None,
        }
    }
}

/// Given the current selected and hovered meshes and provided materials, update the meshes with the
/// appropriate materials...
pub fn pick_highlighting(
    // Resources
    pick_state: Res<PickState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    highlight_params: Res<PickHighlightParams>,
    // Queries
    mut query_selected: Query<(&SelectablePickMesh, &Handle<StandardMaterial>)>,
    mut query_picked: Query<(
        &mut HighlightablePickMesh,
        &PickableMesh,
        &Handle<StandardMaterial>,
        Entity,
    )>,
    query_selectables: Query<&SelectablePickMesh>,
) {
    // Query selectable entities that have changed
    for (selectable, material_handle) in &mut query_selected.iter_mut() {
        let current_color = &mut materials.get_mut(material_handle).unwrap().albedo;
        if selectable.selected() {
            *current_color = highlight_params.selection_color;
        }
    }

    // Query highlightable entities that have changed
    for (mut highlightable, _pickable, material_handle, entity) in &mut query_picked.iter_mut() {
        let current_color = &mut materials.get_mut(material_handle).unwrap().albedo;
        let initial_color = match highlightable.initial_color {
            None => {
                highlightable.initial_color = Some(*current_color);
                *current_color
            }
            Some(color) => color,
        };
        let mut topmost = false;
        for (_group, pick) in pick_state.top_all() {
            if pick.entity == entity {
                topmost = true;
                break;
            }
        }
        if topmost {
            *current_color = highlight_params.hover_color;
        } else if let Ok(selectable) = query_selectables.get(entity) {
            if selectable.selected() {
                *current_color = highlight_params.selection_color;
            } else {
                *current_color = initial_color;
            }
        } else {
            *current_color = initial_color;
        }
    }
}
