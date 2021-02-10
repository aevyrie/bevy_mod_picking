use super::selection::*;
use bevy::{prelude::*, render::color::Color};

#[derive(Clone, Debug, Default)]
pub struct PickableButton {
    initial_material: Handle<StandardMaterial>,
}

pub struct MeshButtonMaterials {
    hovered: Handle<StandardMaterial>,
    pressed: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}

impl FromResources for MeshButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<StandardMaterial>>().unwrap();
        MeshButtonMaterials {
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

pub fn get_initial_mesh_button_material(
    mut query: Query<(&mut PickableButton, &Handle<StandardMaterial>), Added<PickableButton>>,
) {
    for (mut button, material) in query.iter_mut() {
        button.initial_material = material.clone();
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_highlighting(
    button_materials: Res<MeshButtonMaterials>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<StandardMaterial>,
            Option<&Selection>,
            &PickableButton,
        ),
        Or<(Mutated<Interaction>, Mutated<Selection>)>,
    >,
) {
    for (interaction, mut material, selection, button) in interaction_query.iter_mut() {
        *material = match *interaction {
            Interaction::Clicked => button_materials.pressed.clone(),
            Interaction::Hovered => button_materials.hovered.clone(),
            Interaction::None => {
                if let Some(selection) = selection {
                    if selection.selected() {
                        button_materials.selected.clone()
                    } else {
                        button.initial_material.clone()
                    }
                } else {
                    button.initial_material.clone()
                }
            }
        }
    }
}
