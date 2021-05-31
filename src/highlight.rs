use super::selection::*;
use bevy::{prelude::*, render::color::Color};

#[derive(Clone, Debug, Default)]
pub struct PickableButton {
    initial: Option<Handle<StandardMaterial>>,
    hovered: Option<Handle<StandardMaterial>>,
    pressed: Option<Handle<StandardMaterial>>,
    selected: Option<Handle<StandardMaterial>>,
}

pub struct MeshButtonMaterials {
    pub hovered: Handle<StandardMaterial>,
    pub pressed: Handle<StandardMaterial>,
    pub selected: Handle<StandardMaterial>,
}

impl FromWorld for MeshButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("Failed to get resource");
        MeshButtonMaterials {
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

pub fn get_initial_mesh_button_material(
    mut query: Query<(&mut PickableButton, &Handle<StandardMaterial>)>,
) {
    for (mut button, material) in query.iter_mut() {
        if let None = button.initial {
            button.initial = Some(material.clone());
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_highlighting(
    global_button_materials: Res<MeshButtonMaterials>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<StandardMaterial>,
            Option<&Selection>,
            &PickableButton,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) {
    for (interaction, mut material, selection, button) in interaction_query.iter_mut() {
        let try_material = match *interaction {
            Interaction::Clicked => {
                if let Some(button_material) = &button.pressed {
                    Some(button_material.clone())
                } else {
                    Some(global_button_materials.pressed.clone())
                }
            }
            Interaction::Hovered => {
                if let Some(button_material) = &button.hovered {
                    Some(button_material.clone())
                } else {
                    Some(global_button_materials.hovered.clone())
                }
            }
            Interaction::None => {
                if let Some(selection) = selection {
                    if selection.selected() {
                        if let Some(button_material) = &button.selected {
                            Some(button_material.clone())
                        } else {
                            Some(global_button_materials.selected.clone())
                        }
                    } else {
                        button.initial.clone()
                    }
                } else {
                    button.initial.clone()
                }
            }
        };

        if let Some(m) = try_material {
            *material = m;
        } else {
            warn!("Selectable entity missing its initial material");
        }
    }
}
