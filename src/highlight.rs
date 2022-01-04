use super::selection::*;
use crate::PausedForBlockers;
use bevy::{asset::Asset, ecs::system::Resource, prelude::*, render::color::Color};

pub trait IsPickableButton: Component + Clone + std::fmt::Debug + Default {
    type HighlightComponent: Asset + Default;
    fn initial(&self) -> &Option<Handle<Self::HighlightComponent>>;
    fn hovered(&self) -> &Option<Handle<Self::HighlightComponent>>;
    fn pressed(&self) -> &Option<Handle<Self::HighlightComponent>>;
    fn selected(&self) -> &Option<Handle<Self::HighlightComponent>>;
    fn initial_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>>;
    fn hovered_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>>;
    fn pressed_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>>;
    fn selected_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>>;
}

pub trait DefaultButtonMatl<T: Asset + Default>: FromWorld + Resource {
    fn hovered(&self) -> &Handle<T>;
    fn pressed(&self) -> &Handle<T>;
    fn selected(&self) -> &Handle<T>;
}

/// A default highlightable button material implementation of the [`IsPickableButton`] trait that
/// uses bevy's [`StandardMaterial`] for highlighting meshes. You may want to implement your own
/// component if the pickable object being rendered is not a mesh, or doesn't use the
/// `StandardMaterial` component for rendered appearance.
#[derive(Component, Clone, Debug, Default)]
pub struct PickableButton {
    pub initial: Option<Handle<StandardMaterial>>,
    pub hovered: Option<Handle<StandardMaterial>>,
    pub pressed: Option<Handle<StandardMaterial>>,
    pub selected: Option<Handle<StandardMaterial>>,
}
impl IsPickableButton for PickableButton {
    type HighlightComponent = StandardMaterial;

    fn initial_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>> {
        &mut self.initial
    }

    fn hovered_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>> {
        &mut self.hovered
    }

    fn pressed_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>> {
        &mut self.pressed
    }

    fn selected_mut(&mut self) -> &mut Option<Handle<Self::HighlightComponent>> {
        &mut self.selected
    }

    fn initial(&self) -> &Option<Handle<Self::HighlightComponent>> {
        &self.initial
    }

    fn hovered(&self) -> &Option<Handle<Self::HighlightComponent>> {
        &self.hovered
    }

    fn pressed(&self) -> &Option<Handle<Self::HighlightComponent>> {
        &self.pressed
    }

    fn selected(&self) -> &Option<Handle<Self::HighlightComponent>> {
        &self.selected
    }
}

pub struct MeshButtonMaterials {
    pub hovered: Handle<StandardMaterial>,
    pub pressed: Handle<StandardMaterial>,
    pub selected: Handle<StandardMaterial>,
}
impl DefaultButtonMatl<StandardMaterial> for MeshButtonMaterials {
    fn hovered(&self) -> &Handle<StandardMaterial> {
        &self.hovered
    }

    fn pressed(&self) -> &Handle<StandardMaterial> {
        &self.pressed
    }

    fn selected(&self) -> &Handle<StandardMaterial> {
        &self.selected
    }
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

pub fn get_initial_mesh_button_material<T: IsPickableButton>(
    mut query: Query<(&mut T, &Handle<T::HighlightComponent>)>,
) where
    T: Component + Clone + std::fmt::Debug + Default,
{
    for (mut button, material) in query.iter_mut() {
        if button.initial_mut().is_none() {
            *button.initial_mut() = Some(material.clone());
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_highlighting<T: IsPickableButton, U: DefaultButtonMatl<T::HighlightComponent>>(
    paused: Option<Res<PausedForBlockers>>,
    global_button_materials: Res<U>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<T::HighlightComponent>,
            Option<&Selection>,
            &T,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) where
    T: Component + Clone + std::fmt::Debug + Default,
{
    // Set non-hovered material when picking is paused (e.g. while hovering a picking blocker).
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (_, mut material, selection, button) in interaction_query.iter_mut() {
                let try_material = if let Some(selection) = selection {
                    if selection.selected() {
                        if let Some(button_material) = &button.selected() {
                            Some(button_material.clone())
                        } else {
                            Some(global_button_materials.selected().clone())
                        }
                    } else {
                        button.initial().clone()
                    }
                } else {
                    button.initial().clone()
                };
                if let Some(m) = try_material {
                    *material = m;
                } else {
                    warn!("Selectable entity missing its initial material");
                }
            }
            return;
        }
    }
    for (interaction, mut material, selection, button) in interaction_query.iter_mut() {
        let try_material = match *interaction {
            Interaction::Clicked => {
                if let Some(button_material) = &button.pressed() {
                    Some(button_material.clone())
                } else {
                    Some(global_button_materials.pressed().clone())
                }
            }
            Interaction::Hovered => {
                if let Some(button_material) = &button.hovered() {
                    Some(button_material.clone())
                } else {
                    Some(global_button_materials.hovered().clone())
                }
            }
            Interaction::None => {
                if let Some(selection) = selection {
                    if selection.selected() {
                        if let Some(button_material) = &button.selected() {
                            Some(button_material.clone())
                        } else {
                            Some(global_button_materials.selected().clone())
                        }
                    } else {
                        button.initial().clone()
                    }
                } else {
                    button.initial().clone()
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
