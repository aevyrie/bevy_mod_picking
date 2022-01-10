use std::marker::PhantomData;

use super::selection::*;
use crate::PausedForBlockers;
use bevy::{asset::Asset, prelude::*, render::color::Color};

/// A default highlightable button material implementation of the [`IsPickableButton`] trait that
/// uses bevy's [`StandardMaterial`] for highlighting meshes. You may want to implement your own
/// component if the pickable object being rendered is not a mesh, or doesn't use the
/// `StandardMaterial` component for rendered appearance.
#[derive(Component, Clone, Debug, Default)]
pub struct PickableButton<T: Asset> {
    pub initial: Option<Handle<T>>,
    pub hovered: Option<Handle<T>>,
    pub pressed: Option<Handle<T>>,
    pub selected: Option<Handle<T>>,
}

pub struct MeshButtonMaterials<T: Asset, U: FromWorldHelper<T> + ?Sized> {
    pub phantom: PhantomData<U>,
    pub hovered: Handle<T>,
    pub pressed: Handle<T>,
    pub selected: Handle<T>,
}

pub trait FromWorldHelper<T: Asset>: Default {
    fn from_world_helper(world: &mut World) -> MeshButtonMaterials<T, Self>;
}

#[derive(Default)]
pub struct StandardMaterialPickingColors;
impl FromWorldHelper<StandardMaterial> for StandardMaterialPickingColors {
    fn from_world_helper(world: &mut World) -> MeshButtonMaterials<StandardMaterial, Self> {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("Failed to get resource");
        MeshButtonMaterials {
            phantom: PhantomData::default(),
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

impl<T: Asset, U: FromWorldHelper<T>> FromWorld for MeshButtonMaterials<T, U> {
    fn from_world(world: &mut World) -> Self {
        U::from_world_helper(world)
    }
}

#[allow(clippy::type_complexity)]
pub fn get_initial_mesh_button_material<T: Asset>(
    mut query: Query<
        (&mut PickableButton<T>, &Handle<T>),
        Or<(Added<PickableButton<T>>, Changed<Handle<T>>)>,
    >,
) {
    for (mut button, material) in query.iter_mut() {
        if button.initial.is_none() {
            button.initial = Some(material.clone());
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_highlighting<T: Asset, U: 'static + FromWorldHelper<T> + Send + Sync>(
    paused: Option<Res<PausedForBlockers>>,
    global_button_materials: Res<MeshButtonMaterials<T, U>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<T>,
            Option<&Selection>,
            &PickableButton<T>,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) {
    // Set non-hovered material when picking is paused (e.g. while hovering a picking blocker).
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (_, mut material, selection, button) in interaction_query.iter_mut() {
                let try_material = if let Some(selection) = selection {
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
