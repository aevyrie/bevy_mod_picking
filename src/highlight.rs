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
    // The pick group to use for determining highlight state
    group: Group,
}

impl HighlightablePickMesh {
    pub fn new(group: Group) -> Self {
        HighlightablePickMesh{
            group,
            ..Default::default()
        }
    }
}

impl Default for HighlightablePickMesh {
    fn default() -> Self {
        HighlightablePickMesh {
            initial_color: None,
            group: Group::default(),
        }
    }
}

/// Applies highlight an selection color to HighlightablePickMesh entities. Uses the group specified
/// in the component.
pub fn pick_highlighting(
    // Resources
    mut materials: ResMut<Assets<StandardMaterial>>,
    highlight_params: Res<PickHighlightParams>,
    // Queries
    mut query_selected: Query<(
        &mut HighlightablePickMesh,
        Option<&SelectablePickMesh>, // Optional to work with non-selectable entities
        &Handle<StandardMaterial>,
        &PickableMesh,
    )>,
) {
    for (mut highlightable, selectable, material_handle, pickable) in &mut query_selected.iter() {
        let group = highlightable.group;
        let hovered = *pickable.topmost(&group).unwrap();
        let current_color = &mut materials.get_mut(material_handle).unwrap().albedo;
        // If the initial color hasn't been set, we should set it now.
        let initial_color = match highlightable.initial_color {
            None => {
                highlightable.initial_color = Some(*current_color);
                *current_color
            }
            Some(color) => color,
        };
        // When the color is no longer highlighted, the new color depends on selection state. If the
        // entity is selected, the color should be selection color, otherwise it should be the
        // entity's initial color.
        let unhighlight_color = match selectable {
            Some(selectable) => {
                if selectable.selected(&group) {
                    highlight_params.selection_color
                } else {
                    initial_color
                }
            }
            None => initial_color,
        };
        // Update the current entity's color based on selection and highlight state
        *current_color = match pickable.event(&group).unwrap() {
            PickEvents::None => {
                // This is needed when the user clicks elsewhere and the selection state changes.
                // Otherwise, the color would only change after a JustEntered or JustExited.
                // In a more complex example, this might be handled only if
                if hovered {
                    continue;
                } else {
                    unhighlight_color
                }
            }
            PickEvents::JustEntered => highlight_params.hover_color,
            PickEvents::JustExited => unhighlight_color,
        };
    }
}
